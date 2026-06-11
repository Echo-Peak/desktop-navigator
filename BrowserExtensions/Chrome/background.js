import { retryPorts, wsUrl, DEFAULT_HOST, RETRY_DELAY_MS } from "./lib/ports.js";

const ports = retryPorts();
let portIndex = 0;
let socket = null;
let recordingSession = null;
const eventQueue = [];

function send(obj) {
  if (socket && socket.readyState === WebSocket.OPEN) {
    socket.send(JSON.stringify(obj));
  }
}

function sendOrQueue(obj) {
  if (socket && socket.readyState === WebSocket.OPEN) {
    socket.send(JSON.stringify(obj));
  } else {
    eventQueue.push(obj);
  }
}

function flushQueue() {
  while (eventQueue.length && socket && socket.readyState === WebSocket.OPEN) {
    socket.send(JSON.stringify(eventQueue.shift()));
  }
}

async function relayControlToContent(control) {
  try {
    const [tab] = await chrome.tabs.query({
      active: true,
      lastFocusedWindow: true
    });
    if (tab) {
      await chrome.tabs.sendMessage(tab.id, { recorderControl: control });
    }
  } catch (e) {
    /* tab may be gone */
  }
}

chrome.runtime.onMessage.addListener((msg, sender, sendResponse) => {
  if (!msg || !msg.recorder) {
    return false;
  }
  if (msg.recorder === "session") {
    recordingSession = msg.sessionId;
    send({ msg: "ready", sessionId: msg.sessionId });
  } else if (msg.recorder === "event") {
    sendOrQueue({ msg: "event", payload: msg.event });
  }
  sendResponse({ ok: true });
  return true;
});

async function getTargetTabId(msg) {
  if (msg.tabId) {
    return msg.tabId;
  }
  const [tab] = await chrome.tabs.query({ active: true, lastFocusedWindow: true });
  if (!tab) {
    throw new Error("no active tab available");
  }
  return tab.id;
}

async function tabUpdate(msg, props) {
  const id = await getTargetTabId(msg);
  await chrome.tabs.update(id, props);
  return { tabId: id };
}

async function listTabs() {
  const tabs = await chrome.tabs.query({});
  return tabs.map((t) => ({
    tabId: t.id,
    windowId: t.windowId,
    url: t.url,
    title: t.title,
    active: t.active,
    index: t.index
  }));
}

async function activateTab(tabId) {
  const tab = await chrome.tabs.update(tabId, { active: true });
  await chrome.windows.update(tab.windowId, { focused: true });
  return { tabId: tab.id, windowId: tab.windowId };
}

async function relayToContent(msg) {
  const id = await getTargetTabId(msg);
  const res = await chrome.tabs.sendMessage(id, {
    cmd: msg.cmd,
    params: msg.params || {}
  });
  if (!res) {
    throw new Error("no response from content script");
  }
  if (!res.ok) {
    throw new Error(res.error || "content script error");
  }
  return res.result;
}

async function dispatch(msg) {
  switch (msg.cmd) {
    case "navigate":
      return tabUpdate(msg, { url: msg.params.url });
    case "reload":
      await chrome.tabs.reload(await getTargetTabId(msg));
      return { ok: true };
    case "goBack":
      await chrome.tabs.goBack(await getTargetTabId(msg));
      return { ok: true };
    case "goForward":
      await chrome.tabs.goForward(await getTargetTabId(msg));
      return { ok: true };
    case "listTabs":
      return listTabs();
    case "activateTab":
      return activateTab(msg.params.tabId);
    default:
      return relayToContent(msg);
  }
}

async function handleMessage(raw) {
  let msg;
  try {
    msg = JSON.parse(raw);
  } catch (e) {
    return;
  }
  if (msg && typeof msg.msg === "string") {
    if (msg.msg === "stop") {
      recordingSession = null;
      await relayControlToContent("stop");
    } else if (msg.msg === "pause") {
      await relayControlToContent("pause");
    } else if (msg.msg === "start") {
      await relayControlToContent("resume");
    }
    return;
  }
  if (!msg || typeof msg.id === "undefined") {
    return;
  }
  try {
    const result = await dispatch(msg);
    send({ id: msg.id, ok: true, result });
  } catch (e) {
    send({ id: msg.id, ok: false, error: e && e.message ? e.message : String(e) });
  }
}

function scheduleReconnect() {
  portIndex = (portIndex + 1) % ports.length;
  setTimeout(connect, RETRY_DELAY_MS);
}

function connect() {
  try {
    socket = new WebSocket(wsUrl(DEFAULT_HOST, ports[portIndex]));
  } catch (e) {
    scheduleReconnect();
    return;
  }
  socket.addEventListener("open", () => {
    send({ event: "ready" });
    if (recordingSession) {
      send({ msg: "ready", sessionId: recordingSession });
    }
    flushQueue();
  });
  socket.addEventListener("message", (ev) => {
    handleMessage(ev.data);
  });
  socket.addEventListener("close", () => {
    scheduleReconnect();
  });
  socket.addEventListener("error", () => {
    try {
      socket.close();
    } catch (e) {
      scheduleReconnect();
    }
  });
}

connect();
