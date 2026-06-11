use std::collections::BTreeMap;
use std::collections::HashMap;
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::mpsc::{Receiver as SyncReceiver, Sender as SyncSender};
use std::sync::{Arc, Mutex};
use std::time::Duration;

use futures_util::{SinkExt, StreamExt};
use navigator_core::engine::{
    BrowserManager, DomBridge, DomRect, EngineError, EngineResult, WindowGeometry,
};
use navigator_core::schema::{ExtractField, QueryTarget};
use serde_json::{json, Value};
use tokio::net::TcpListener;
use tokio::sync::mpsc::{unbounded_channel, UnboundedReceiver, UnboundedSender};
use tokio_tungstenite::tungstenite::Message;

const BASE_PORT: u16 = 35282;
const MAX_EXTRA_PORTS: u16 = 10;
const DEFAULT_TIMEOUT_MS: u64 = 8000;

struct Reply {
    ok: bool,
    result: Value,
    error: String,
}

type Pending = Arc<Mutex<HashMap<u64, SyncSender<Reply>>>>;
type RecorderSink = SyncSender<Value>;
type Session = Arc<Mutex<Option<String>>>;

#[derive(Clone)]
pub struct BridgeHandle {
    out_tx: UnboundedSender<Value>,
    pending: Pending,
    next_id: Arc<AtomicU64>,
    ready: Arc<AtomicBool>,
    session: Session,
    port: u16,
}

impl BridgeHandle {
    pub fn port(&self) -> u16 {
        self.port
    }

    pub fn is_ready(&self) -> bool {
        self.ready.load(Ordering::SeqCst)
    }

    pub fn set_recording_session(&self, session_id: Option<String>) {
        if let Ok(mut s) = self.session.lock() {
            *s = session_id;
        }
    }

    pub fn send(&self, value: Value) {
        let _ = self.out_tx.send(value);
    }

    pub fn request(&self, cmd: &str, params: Value, timeout_ms: u64) -> EngineResult<Value> {
        let id = self.next_id.fetch_add(1, Ordering::SeqCst);
        let (tx, rx): (SyncSender<Reply>, SyncReceiver<Reply>) = std::sync::mpsc::channel();
        self.pending
            .lock()
            .map_err(|_| EngineError::Bridge("pending lock poisoned".into()))?
            .insert(id, tx);

        let payload = json!({ "id": id, "cmd": cmd, "params": params });
        self.out_tx
            .send(payload)
            .map_err(|_| EngineError::Bridge("bridge transport closed".into()))?;

        match rx.recv_timeout(Duration::from_millis(timeout_ms)) {
            Ok(reply) if reply.ok => Ok(reply.result),
            Ok(reply) => Err(EngineError::Bridge(reply.error)),
            Err(_) => {
                if let Ok(mut p) = self.pending.lock() {
                    p.remove(&id);
                }
                Err(EngineError::Timeout(format!(
                    "no bridge response for '{cmd}' within {timeout_ms}ms"
                )))
            }
        }
    }
}

pub fn start() -> std::io::Result<(BridgeHandle, std::sync::mpsc::Receiver<Value>)> {
    let (out_tx, out_rx) = unbounded_channel::<Value>();
    let pending: Pending = Arc::new(Mutex::new(HashMap::new()));
    let ready = Arc::new(AtomicBool::new(false));
    let session: Session = Arc::new(Mutex::new(None));
    let (recorder_tx, recorder_rx) = std::sync::mpsc::channel::<Value>();

    let (port_tx, port_rx) = std::sync::mpsc::channel::<u16>();
    let pending_task = pending.clone();
    let ready_task = ready.clone();
    let session_task = session.clone();

    std::thread::spawn(move || {
        let rt = match tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
        {
            Ok(rt) => rt,
            Err(_) => return,
        };
        rt.block_on(async move {
            let listener = match bind_first().await {
                Some(l) => l,
                None => return,
            };
            let port = listener.local_addr().map(|a| a.port()).unwrap_or(BASE_PORT);
            let _ = port_tx.send(port);
            serve(
                listener,
                out_rx,
                pending_task,
                ready_task,
                recorder_tx,
                session_task,
            )
            .await;
        });
    });

    let port = port_rx
        .recv_timeout(Duration::from_secs(5))
        .map_err(|_| std::io::Error::new(std::io::ErrorKind::AddrInUse, "no free bridge port"))?;

    Ok((
        BridgeHandle {
            out_tx,
            pending,
            next_id: Arc::new(AtomicU64::new(1)),
            ready,
            session,
            port,
        },
        recorder_rx,
    ))
}

async fn bind_first() -> Option<TcpListener> {
    for offset in 0..=MAX_EXTRA_PORTS {
        let port = BASE_PORT + offset;
        if let Ok(listener) = TcpListener::bind(("127.0.0.1", port)).await {
            return Some(listener);
        }
    }
    None
}

async fn serve(
    listener: TcpListener,
    mut out_rx: UnboundedReceiver<Value>,
    pending: Pending,
    ready: Arc<AtomicBool>,
    recorder: RecorderSink,
    session: Session,
) {
    loop {
        let (stream, _) = match listener.accept().await {
            Ok(v) => v,
            Err(_) => continue,
        };
        let ws = match tokio_tungstenite::accept_async(stream).await {
            Ok(ws) => ws,
            Err(_) => continue,
        };
        let (mut write, mut read) = ws.split();
        ready.store(false, Ordering::SeqCst);

        loop {
            tokio::select! {
                outgoing = out_rx.recv() => {
                    match outgoing {
                        Some(value) => {
                            if write.send(Message::Text(value.to_string())).await.is_err() {
                                break;
                            }
                        }
                        None => return,
                    }
                }
                incoming = read.next() => {
                    match incoming {
                        Some(Ok(Message::Text(text))) => {
                            handle_incoming(&text, &pending, &ready, &recorder, &session)
                        }
                        Some(Ok(Message::Close(_))) | None => break,
                        Some(Err(_)) => break,
                        _ => {}
                    }
                }
            }
        }

        ready.store(false, Ordering::SeqCst);
    }
}

fn handle_recorder(msg: &Value, recorder: &RecorderSink, session: &Session) {
    let kind = msg.get("msg").and_then(|m| m.as_str()).unwrap_or("");
    let expected = session.lock().ok().and_then(|s| s.clone());

    match kind {
        "ready" => {
            let sid = msg.get("sessionId").and_then(|s| s.as_str());
            if let (Some(sid), Some(exp)) = (sid, expected.as_deref()) {
                if sid == exp {
                    let _ = recorder.send(json!({ "kind": "ready", "sessionId": sid }));
                }
            }
        }
        "event" => {
            let payload = match msg.get("payload") {
                Some(p) => p,
                None => return,
            };
            let sid = payload.get("sessionId").and_then(|s| s.as_str());
            match (sid, expected.as_deref()) {
                (Some(sid), Some(exp)) if sid == exp => {
                    let _ = recorder.send(json!({ "kind": "event", "payload": payload }));
                }
                _ => {}
            }
        }
        "browser_closed" => {
            let _ = recorder.send(json!({ "kind": "closed" }));
        }
        _ => {}
    }
}

fn handle_incoming(
    text: &str,
    pending: &Pending,
    ready: &Arc<AtomicBool>,
    recorder: &RecorderSink,
    session: &Session,
) {
    let msg: Value = match serde_json::from_str(text) {
        Ok(v) => v,
        Err(_) => return,
    };

    if msg.get("event").and_then(|e| e.as_str()) == Some("ready") {
        ready.store(true, Ordering::SeqCst);
        return;
    }

    if msg.get("msg").is_some() {
        handle_recorder(&msg, recorder, session);
        return;
    }

    let id = match msg.get("id").and_then(|i| i.as_u64()) {
        Some(id) => id,
        None => return,
    };

    let sender = pending.lock().ok().and_then(|mut p| p.remove(&id));
    if let Some(tx) = sender {
        let ok = msg.get("ok").and_then(|v| v.as_bool()).unwrap_or(false);
        let _ = tx.send(Reply {
            ok,
            result: msg.get("result").cloned().unwrap_or(Value::Null),
            error: msg
                .get("error")
                .and_then(|e| e.as_str())
                .unwrap_or("bridge error")
                .to_string(),
        });
    }
}

fn screen_from(value: &Value) -> EngineResult<(DomRect, WindowGeometry)> {
    let rect_val = value
        .get("rect")
        .ok_or_else(|| EngineError::Bridge("missing rect in geometry".into()))?;
    let rect: DomRect = serde_json::from_value(rect_val.clone())
        .map_err(|e| EngineError::Bridge(format!("bad rect: {e}")))?;
    let w = value
        .get("window")
        .ok_or_else(|| EngineError::Bridge("missing window in geometry".into()))?;
    let window = WindowGeometry {
        screen_x: w.get("screenX").and_then(|v| v.as_f64()).unwrap_or(0.0),
        screen_y: w.get("screenY").and_then(|v| v.as_f64()).unwrap_or(0.0),
        chrome_height: w.get("chromeHeight").and_then(|v| v.as_f64()).unwrap_or(0.0),
    };
    Ok((rect, window))
}

impl BrowserManager for BridgeHandle {
    fn navigate(&mut self, url: &str) -> EngineResult<()> {
        self.request("navigate", json!({ "url": url }), DEFAULT_TIMEOUT_MS)?;
        Ok(())
    }
    fn reload(&mut self) -> EngineResult<()> {
        self.request("reload", json!({}), DEFAULT_TIMEOUT_MS)?;
        Ok(())
    }
    fn go_back(&mut self) -> EngineResult<()> {
        self.request("goBack", json!({}), DEFAULT_TIMEOUT_MS)?;
        Ok(())
    }
    fn go_forward(&mut self) -> EngineResult<()> {
        self.request("goForward", json!({}), DEFAULT_TIMEOUT_MS)?;
        Ok(())
    }
}

impl DomBridge for BridgeHandle {
    fn element_geometry(&mut self, selector: &str) -> EngineResult<(DomRect, WindowGeometry)> {
        let res = self.request("getRect", json!({ "selector": selector }), DEFAULT_TIMEOUT_MS)?;
        screen_from(&res)
    }

    fn get_text(&mut self, selector: &str) -> EngineResult<String> {
        let res = self.request("getText", json!({ "selector": selector }), DEFAULT_TIMEOUT_MS)?;
        Ok(res.as_str().unwrap_or_default().to_string())
    }

    fn get_attribute(&mut self, selector: &str, attribute: &str) -> EngineResult<String> {
        let res = self.request(
            "getAttribute",
            json!({ "selector": selector, "attribute": attribute }),
            DEFAULT_TIMEOUT_MS,
        )?;
        Ok(res.as_str().unwrap_or_default().to_string())
    }

    fn query_property(
        &mut self,
        target: QueryTarget,
        selector: Option<&str>,
        property: &str,
    ) -> EngineResult<Value> {
        let target = serde_json::to_value(target)
            .map_err(|e| EngineError::Bridge(format!("bad target: {e}")))?;
        self.request(
            "queryProperty",
            json!({ "target": target, "selector": selector, "property": property }),
            DEFAULT_TIMEOUT_MS,
        )
    }

    fn is_visible(&mut self, selector: &str) -> EngineResult<bool> {
        let res = self.request(
            "isVisible",
            json!({ "selector": selector }),
            DEFAULT_TIMEOUT_MS,
        )?;
        Ok(res.as_bool().unwrap_or(false))
    }

    fn extract_collection(
        &mut self,
        container_selector: &str,
        item_selector: &str,
        fields: &BTreeMap<String, ExtractField>,
    ) -> EngineResult<Value> {
        let fields = serde_json::to_value(fields)
            .map_err(|e| EngineError::Bridge(format!("bad fields: {e}")))?;
        self.request(
            "extractCollection",
            json!({
                "containerSelector": container_selector,
                "itemSelector": item_selector,
                "fields": fields
            }),
            DEFAULT_TIMEOUT_MS,
        )
    }
}
