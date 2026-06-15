mod adapters;
mod bridge;
mod update;

use std::path::PathBuf;
use std::process::Child;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};
use std::time::Duration;

use adapters::{ChromeLauncher, EnigoSink, KeyringResolver};
use bridge::BridgeHandle;
use navigator_core::data::now_ms;
use navigator_core::engine::{BrowserManager, Engine, SystemClock, UserClient};
use navigator_core::schema::{AutomationAction, BrowserContext, PageContext};
use navigator_core::AppPaths;
use serde::Serialize;
use serde_json::{json, Value};
use tauri::{AppHandle, Emitter, Manager, State};

const KEYRING_SERVICE: &str = "app.desktop-navigator";

type ChildRegistry = Arc<Mutex<Vec<Child>>>;

struct AppState {
    bridge: BridgeHandle,
    children: ChildRegistry,
    cycle_active: Arc<AtomicBool>,
}

#[derive(Serialize)]
pub struct AppDirs {
    pub install_dir: String,
    pub updates_dir: String,
    pub logs_dir: String,
    pub pages_dir: String,
    pub captcha_resolvers_dir: String,
}

#[tauri::command]
fn app_dirs() -> Result<AppDirs, String> {
    let paths = AppPaths::resolve().map_err(|e| e.to_string())?;
    paths.ensure_base_dirs().map_err(|e| e.to_string())?;
    paths
        .ensure_captcha_dir(update::CURRENT_VERSION)
        .map_err(|e| e.to_string())?;
    Ok(AppDirs {
        install_dir: paths.install_dir().display().to_string(),
        updates_dir: paths.updates_dir().display().to_string(),
        logs_dir: paths.logs_dir().display().to_string(),
        pages_dir: paths.pages_dir().display().to_string(),
        captcha_resolvers_dir: paths
            .captcha_resolvers_dir(update::CURRENT_VERSION)
            .display()
            .to_string(),
    })
}

fn sanitize(name: &str) -> String {
    name.chars()
        .map(|c| if c.is_ascii_alphanumeric() || c == '-' || c == '_' || c == '.' { c } else { '_' })
        .collect()
}

#[tauri::command]
fn list_pages() -> Result<Vec<String>, String> {
    let paths = AppPaths::resolve().map_err(|e| e.to_string())?;
    let dir = paths.pages_dir();
    if !dir.exists() {
        return Ok(Vec::new());
    }
    let mut names = Vec::new();
    for entry in std::fs::read_dir(&dir).map_err(|e| e.to_string())? {
        let entry = entry.map_err(|e| e.to_string())?;
        let path = entry.path();
        if path.extension().and_then(|e| e.to_str()) == Some("json") {
            if let Some(stem) = path.file_stem().and_then(|s| s.to_str()) {
                names.push(stem.to_string());
            }
        }
    }
    names.sort();
    Ok(names)
}

#[tauri::command]
fn read_page(name: String) -> Result<String, String> {
    let paths = AppPaths::resolve().map_err(|e| e.to_string())?;
    let file = paths.pages_dir().join(format!("{}.json", sanitize(&name)));
    std::fs::read_to_string(file).map_err(|e| e.to_string())
}

#[tauri::command]
fn write_page(name: String, json: String) -> Result<(), String> {
    let paths = AppPaths::resolve().map_err(|e| e.to_string())?;
    let dir = paths.pages_dir();
    std::fs::create_dir_all(&dir).map_err(|e| e.to_string())?;
    let file = dir.join(format!("{}.json", sanitize(&name)));
    std::fs::write(file, json).map_err(|e| e.to_string())
}

#[tauri::command]
fn delete_page(name: String) -> Result<(), String> {
    let paths = AppPaths::resolve().map_err(|e| e.to_string())?;
    let file = paths.pages_dir().join(format!("{}.json", sanitize(&name)));
    if file.exists() {
        std::fs::remove_file(file).map_err(|e| e.to_string())?;
    }
    Ok(())
}

fn captcha_dir(paths: &AppPaths) -> PathBuf {
    paths.captcha_resolvers_dir(update::CURRENT_VERSION)
}

#[tauri::command]
fn list_captcha_resolvers() -> Result<Vec<String>, String> {
    let paths = AppPaths::resolve().map_err(|e| e.to_string())?;
    let dir = captcha_dir(&paths);
    if !dir.exists() {
        return Ok(Vec::new());
    }
    let mut names = Vec::new();
    for entry in std::fs::read_dir(&dir).map_err(|e| e.to_string())? {
        let entry = entry.map_err(|e| e.to_string())?;
        let path = entry.path();
        if path.extension().and_then(|e| e.to_str()) == Some("json") {
            if let Some(stem) = path.file_stem().and_then(|s| s.to_str()) {
                names.push(stem.to_string());
            }
        }
    }
    names.sort();
    Ok(names)
}

#[tauri::command]
fn read_captcha_resolver(id: String) -> Result<String, String> {
    let paths = AppPaths::resolve().map_err(|e| e.to_string())?;
    let file = captcha_dir(&paths).join(format!("{}.json", sanitize(&id)));
    std::fs::read_to_string(file).map_err(|e| e.to_string())
}

#[tauri::command]
fn write_captcha_resolver(id: String, json: String) -> Result<(), String> {
    let paths = AppPaths::resolve().map_err(|e| e.to_string())?;
    let dir = captcha_dir(&paths);
    std::fs::create_dir_all(&dir).map_err(|e| e.to_string())?;
    let file = dir.join(format!("{}.json", sanitize(&id)));
    std::fs::write(file, json).map_err(|e| e.to_string())
}

#[tauri::command]
fn delete_captcha_resolver(id: String) -> Result<(), String> {
    let paths = AppPaths::resolve().map_err(|e| e.to_string())?;
    let file = captcha_dir(&paths).join(format!("{}.json", sanitize(&id)));
    if file.exists() {
        std::fs::remove_file(file).map_err(|e| e.to_string())?;
    }
    Ok(())
}

#[tauri::command]
fn store_secret(key: String, value: String) -> Result<(), String> {
    KeyringResolver::new(KEYRING_SERVICE)
        .store(&key, &value)
        .map_err(|e| e.to_string())
}

#[tauri::command]
fn input_available() -> bool {
    EnigoSink::new().is_ok()
}

#[tauri::command]
fn bridge_status(state: State<AppState>) -> Value {
    json!({
        "port": state.bridge.port(),
        "ready": state.bridge.is_ready()
    })
}

fn extension_dir(app: &AppHandle) -> PathBuf {
    if let Ok(resource) = app.path().resource_dir() {
        let bundled = resource.join("BrowserExtensions").join("Chrome");
        if bundled.join("manifest.json").exists() {
            return bundled;
        }
    }
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("..")
        .join("BrowserExtensions")
        .join("Chrome")
}

fn profile_dir() -> Result<PathBuf, String> {
    let paths = AppPaths::resolve().map_err(|e| e.to_string())?;
    Ok(paths.install_dir().join("profile"))
}

#[tauri::command]
fn launch_session(
    app: AppHandle,
    state: State<AppState>,
    browser_context_json: String,
    url: Option<String>,
) -> Result<(), String> {
    let context: BrowserContext =
        serde_json::from_str(&browser_context_json).map_err(|e| e.to_string())?;
    let launcher = ChromeLauncher::new(extension_dir(&app), profile_dir()?);
    let child = launcher
        .launch(&context.browser_config, url.as_deref())
        .map_err(|e| e.to_string())?;
    state
        .children
        .lock()
        .map_err(|_| "child registry poisoned".to_string())?
        .push(child);
    Ok(())
}

#[tauri::command]
fn end_session(state: State<AppState>) -> Result<(), String> {
    let mut children = state
        .children
        .lock()
        .map_err(|_| "child registry poisoned".to_string())?;
    for mut child in children.drain(..) {
        let _ = child.kill();
    }
    state.cycle_active.store(false, Ordering::SeqCst);
    Ok(())
}

#[tauri::command]
fn run_page(state: State<AppState>, page_json: String) -> Result<(), String> {
    let page: PageContext = serde_json::from_str(&page_json).map_err(|e| e.to_string())?;
    let sink = EnigoSink::new().map_err(|e| e.to_string())?;
    let user = UserClient::new(sink, now_ms() as u64);
    let mut engine = Engine::new(
        state.bridge.clone(),
        state.bridge.clone(),
        user,
        KeyringResolver::new(KEYRING_SERVICE),
        SystemClock,
    );
    engine.run(&page).map_err(|e| e.to_string())
}

fn normalize_url(domain: &str) -> String {
    if domain.starts_with("http://") || domain.starts_with("https://") {
        domain.to_string()
    } else {
        format!("https://{domain}")
    }
}

fn run_pages(bridge: &BridgeHandle, pages: &[PageContext]) -> Result<(), String> {
    for page in pages {
        let starts_with_nav =
            matches!(page.steps.first().map(|s| &s.action), Some(AutomationAction::Navigate { .. }));
        if !starts_with_nav {
            let mut mgr = bridge.clone();
            mgr.navigate(&normalize_url(&page.domain))
                .map_err(|e| e.to_string())?;
        }
        let sink = EnigoSink::new().map_err(|e| e.to_string())?;
        let user = UserClient::new(sink, now_ms() as u64);
        let mut engine = Engine::new(
            bridge.clone(),
            bridge.clone(),
            user,
            KeyringResolver::new(KEYRING_SERVICE),
            SystemClock,
        );
        engine.run(page).map_err(|e| e.to_string())?;
    }
    Ok(())
}

#[tauri::command]
fn run_session(
    app: AppHandle,
    state: State<AppState>,
    browser_context_json: String,
    pages_json: Vec<String>,
    start_url: Option<String>,
) -> Result<(), String> {
    let context: BrowserContext =
        serde_json::from_str(&browser_context_json).map_err(|e| e.to_string())?;
    let pages: Vec<PageContext> = pages_json
        .iter()
        .map(|j| serde_json::from_str::<PageContext>(j))
        .collect::<Result<_, _>>()
        .map_err(|e| e.to_string())?;

    let launcher = ChromeLauncher::new(extension_dir(&app), profile_dir()?);
    let child = launcher
        .launch(&context.browser_config, start_url.as_deref())
        .map_err(|e| e.to_string())?;
    state
        .children
        .lock()
        .map_err(|_| "child registry poisoned".to_string())?
        .push(child);

    let bridge = state.bridge.clone();
    let children = state.children.clone();
    let cycle_active = state.cycle_active.clone();
    let app = app.clone();
    cycle_active.store(true, Ordering::SeqCst);

    std::thread::spawn(move || {
        let mut waited = 0u64;
        while !bridge.is_ready() && waited < 20_000 {
            std::thread::sleep(Duration::from_millis(200));
            waited += 200;
        }

        let result = if bridge.is_ready() {
            run_pages(&bridge, &pages)
        } else {
            Err("browser extension did not connect".to_string())
        };

        if let Ok(mut guard) = children.lock() {
            for mut c in guard.drain(..) {
                let _ = c.kill();
            }
        }
        cycle_active.store(false, Ordering::SeqCst);

        match result {
            Ok(()) => {
                let _ = app.emit("session:done", ());
            }
            Err(err) => {
                let _ = app.emit("session:error", err);
            }
        }
    });

    Ok(())
}

fn random_session_id() -> String {
    let nanos = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_nanos())
        .unwrap_or(0);
    let pid = std::process::id() as u128;
    format!("{:032x}", nanos ^ (pid << 64))
}

#[tauri::command]
fn start_recording(
    app: AppHandle,
    state: State<AppState>,
    browser_context_json: String,
    url: String,
) -> Result<String, String> {
    let context: BrowserContext =
        serde_json::from_str(&browser_context_json).map_err(|e| e.to_string())?;
    let session_id = random_session_id();
    state.bridge.set_recording_session(Some(session_id.clone()));

    let sep = if url.contains('?') { '&' } else { '?' };
    let launch_url = format!("{url}{sep}_bpsession={session_id}");

    let launcher = ChromeLauncher::new(extension_dir(&app), profile_dir()?);
    let child = launcher
        .launch(&context.browser_config, Some(&launch_url))
        .map_err(|e| e.to_string())?;
    state
        .children
        .lock()
        .map_err(|_| "child registry poisoned".to_string())?
        .push(child);
    state.cycle_active.store(true, Ordering::SeqCst);

    Ok(session_id)
}

#[tauri::command]
fn stop_recording(state: State<AppState>) -> Result<(), String> {
    state.bridge.send(json!({ "msg": "stop" }));
    state.bridge.set_recording_session(None);
    let mut children = state
        .children
        .lock()
        .map_err(|_| "child registry poisoned".to_string())?;
    for mut child in children.drain(..) {
        let _ = child.kill();
    }
    state.cycle_active.store(false, Ordering::SeqCst);
    Ok(())
}

#[tauri::command]
fn update_check(app: AppHandle) {
    std::thread::spawn(move || update::check_and_update(&app));
}

#[tauri::command]
fn update_restart(app: AppHandle) {
    app.restart();
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let args: Vec<String> = std::env::args().collect();
    if let Some(version) = update::parse_update_flag(&args) {
        match update::run_helper(&version) {
            Ok(v) => {
                println!("OK {v}");
                std::process::exit(0);
            }
            Err(e) => {
                eprintln!("ERR {e}");
                std::process::exit(1);
            }
        }
    }

    let (bridge, recorder_rx) = bridge::start().expect("failed to start bridge server");
    let cycle_active = Arc::new(AtomicBool::new(false));
    let cycle_for_setup = cycle_active.clone();

    tauri::Builder::default()
        .setup(move |app| {
            let handle = app.handle().clone();
            std::thread::spawn(move || {
                while let Ok(value) = recorder_rx.recv() {
                    let _ = handle.emit("recorder:event", value);
                }
            });

            let update_handle = app.handle().clone();
            let cycle = cycle_for_setup;
            std::thread::spawn(move || loop {
                while cycle.load(Ordering::SeqCst) {
                    std::thread::sleep(Duration::from_secs(30));
                }
                update::check_and_update(&update_handle);
                std::thread::sleep(Duration::from_secs(2 * 60 * 60));
            });
            Ok(())
        })
        .manage(AppState {
            bridge,
            children: Arc::new(Mutex::new(Vec::new())),
            cycle_active,
        })
        .invoke_handler(tauri::generate_handler![
            app_dirs,
            list_pages,
            read_page,
            write_page,
            delete_page,
            list_captcha_resolvers,
            read_captcha_resolver,
            write_captcha_resolver,
            delete_captcha_resolver,
            store_secret,
            input_available,
            bridge_status,
            launch_session,
            end_session,
            run_page,
            run_session,
            start_recording,
            stop_recording,
            update_check,
            update_restart
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
