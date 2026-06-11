function requireElement(selector) {
  const el = document.querySelector(selector);
  if (!el) {
    throw new Error(`no element matches selector: ${selector}`);
  }
  return el;
}

function chromeHeight() {
  const diff = window.outerHeight - window.innerHeight;
  return diff > 0 ? diff : 0;
}

function getRect(selector) {
  const el = requireElement(selector);
  const r = el.getBoundingClientRect();
  return {
    rect: {
      x: r.x,
      y: r.y,
      width: r.width,
      height: r.height,
      top: r.top,
      left: r.left
    },
    window: {
      screenX: window.screenX,
      screenY: window.screenY,
      chromeHeight: chromeHeight()
    }
  };
}

function isVisible(selector) {
  const el = document.querySelector(selector);
  if (!el) {
    return false;
  }
  const r = el.getBoundingClientRect();
  const style = window.getComputedStyle(el);
  return (
    r.width > 0 &&
    r.height > 0 &&
    style.visibility !== "hidden" &&
    style.display !== "none" &&
    style.opacity !== "0"
  );
}

function extractOne(el, field) {
  const target = field.selector ? el.querySelector(field.selector) : el;
  if (!target) {
    return null;
  }
  if (field.extractType === "attribute") {
    return target.getAttribute(field.attributeName);
  }
  return target.innerText;
}

function extractCollection(params) {
  const container = requireElement(params.containerSelector);
  const items = Array.from(container.querySelectorAll(params.itemSelector));
  return items.map((item) => {
    const row = {};
    for (const [key, field] of Object.entries(params.fields || {})) {
      row[key] = extractOne(item, field);
    }
    return row;
  });
}

function queryProperty(params) {
  const base =
    params.target === "document" ? document : requireElement(params.selector);
  const value = base[params.property];
  if (typeof value === "undefined") {
    throw new Error(`property not found: ${params.property}`);
  }
  if (
    typeof value === "string" ||
    typeof value === "number" ||
    typeof value === "boolean"
  ) {
    return value;
  }
  return String(value);
}

function scrollPage(params) {
  if (params.selector) {
    requireElement(params.selector).scrollIntoView({
      behavior: "instant",
      block: "center"
    });
  } else {
    window.scrollBy(params.deltaX || 0, params.deltaY || 0);
  }
  return { scrollX: window.scrollX, scrollY: window.scrollY };
}

function pageDimensions() {
  const doc = document.documentElement;
  return {
    width: window.innerWidth,
    height: window.innerHeight,
    scrollWidth: doc.scrollWidth,
    scrollHeight: doc.scrollHeight
  };
}

function runContent(msg) {
  const p = msg.params || {};
  switch (msg.cmd) {
    case "getTitle":
      return document.title;
    case "getRect":
      return getRect(p.selector);
    case "getText":
      return requireElement(p.selector).innerText;
    case "getAttribute":
      return requireElement(p.selector).getAttribute(p.attribute);
    case "queryProperty":
      return queryProperty(p);
    case "isVisible":
      return isVisible(p.selector);
    case "extractCollection":
      return extractCollection(p);
    case "scroll":
      return scrollPage(p);
    case "pageDimensions":
      return pageDimensions();
    default:
      throw new Error(`unknown content command: ${msg.cmd}`);
  }
}

chrome.runtime.onMessage.addListener((msg, sender, sendResponse) => {
  if (msg && msg.recorderControl) {
    handleControl(msg.recorderControl);
    sendResponse({ ok: true });
    return true;
  }
  try {
    sendResponse({ ok: true, result: runContent(msg) });
  } catch (e) {
    sendResponse({ ok: false, error: e && e.message ? e.message : String(e) });
  }
  return true;
});

const NONPRINTABLE_KEYS = new Set([
  "Enter",
  "Tab",
  "Escape",
  "ArrowUp",
  "ArrowDown",
  "ArrowLeft",
  "ArrowRight",
  "F1",
  "F2",
  "F3",
  "F4",
  "F5",
  "F6",
  "F7",
  "F8",
  "F9",
  "F10",
  "F11",
  "F12"
]);

const recorder = {
  sessionId: new URLSearchParams(location.search).get("_bpsession"),
  paused: false,
  sequence: 0,
  startTime: Date.now(),
  inputTimers: new WeakMap()
};

function uuid() {
  if (crypto && crypto.randomUUID) {
    return crypto.randomUUID();
  }
  return "id-" + Math.random().toString(36).slice(2) + Date.now().toString(36);
}

function stripSession(href) {
  try {
    const u = new URL(href);
    u.searchParams.delete("_bpsession");
    return u.toString();
  } catch (e) {
    return href;
  }
}

function cssEscape(value) {
  if (window.CSS && CSS.escape) {
    return CSS.escape(value);
  }
  return String(value).replace(/[^a-zA-Z0-9_-]/g, "\\$&");
}

function isUnique(selector) {
  try {
    return document.querySelectorAll(selector).length === 1;
  } catch (e) {
    return false;
  }
}

function xpathFor(el) {
  if (el.id) {
    return `//*[@id="${el.id}"]`;
  }
  const parts = [];
  let node = el;
  while (node && node.nodeType === 1 && node !== document.documentElement) {
    let index = 1;
    let sib = node.previousElementSibling;
    while (sib) {
      if (sib.tagName === node.tagName) index += 1;
      sib = sib.previousElementSibling;
    }
    parts.unshift(`${node.tagName.toLowerCase()}[${index}]`);
    node = node.parentElement;
  }
  return "/" + parts.join("/");
}

function ancestorPath(el) {
  const parts = [];
  let node = el;
  while (node && node.nodeType === 1 && node !== document.body) {
    let part = node.tagName.toLowerCase();
    const parent = node.parentElement;
    if (parent) {
      const sameTag = Array.from(parent.children).filter(
        (c) => c.tagName === node.tagName
      );
      if (sameTag.length > 1) {
        part += `:nth-child(${Array.from(parent.children).indexOf(node) + 1})`;
      }
    }
    parts.unshift(part);
    node = parent;
  }
  return parts.join(" > ");
}

function bestSelector(el) {
  const tag = el.tagName.toLowerCase();
  if (el.id) {
    const s = `#${cssEscape(el.id)}`;
    if (isUnique(s)) return s;
  }
  if (el.name) {
    const s = `${tag}[name="${el.name}"]`;
    if (isUnique(s)) return s;
  }
  if (el.classList.length) {
    const s = tag + "." + Array.from(el.classList).map(cssEscape).join(".");
    if (isUnique(s)) return s;
  }
  const aria = el.getAttribute("aria-label");
  if (aria) {
    const s = `${tag}[aria-label="${aria}"]`;
    if (isUnique(s)) return s;
  }
  const placeholder = el.getAttribute("placeholder");
  if (placeholder) {
    const s = `${tag}[placeholder="${placeholder}"]`;
    if (isUnique(s)) return s;
  }
  const path = ancestorPath(el);
  if (path && isUnique(path)) return path;
  return null;
}

function describe(el) {
  const r = el.getBoundingClientRect();
  const text = (el.innerText || "").trim().slice(0, 80);
  const selector = bestSelector(el);
  return {
    tagName: el.tagName,
    id: el.id || undefined,
    classList: Array.from(el.classList),
    name: el.getAttribute("name") || undefined,
    type: el.getAttribute("type") || undefined,
    placeholder: el.getAttribute("placeholder") || undefined,
    ariaLabel: el.getAttribute("aria-label") || undefined,
    text: text || undefined,
    href: el.getAttribute("href") || undefined,
    selector: selector || xpathFor(el),
    xpath: xpathFor(el),
    boundingRect: {
      x: r.x,
      y: r.y,
      width: r.width,
      height: r.height,
      top: r.top,
      right: r.right,
      bottom: r.bottom,
      left: r.left
    }
  };
}

function emit(type, extra) {
  if (!recorder.sessionId || recorder.paused) return;
  recorder.sequence += 1;
  const event = {
    id: uuid(),
    sequence: recorder.sequence,
    sessionId: recorder.sessionId,
    timestamp: Date.now() - recorder.startTime,
    type,
    pageUrl: stripSession(location.href),
    ...extra
  };
  chrome.runtime.sendMessage({ recorder: "event", event });
}

function handleControl(control) {
  if (control === "pause") recorder.paused = true;
  else if (control === "resume") recorder.paused = false;
  else if (control === "stop") {
    recorder.paused = true;
    recorder.sessionId = null;
  }
}

function startRecording() {
  chrome.runtime.sendMessage({
    recorder: "session",
    sessionId: recorder.sessionId
  });

  emit("navigate", {});

  document.addEventListener(
    "click",
    (e) => {
      const el = e.target;
      if (el && el.nodeType === 1) emit("click", { element: describe(el) });
    },
    true
  );

  document.addEventListener(
    "dblclick",
    (e) => {
      const el = e.target;
      if (el && el.nodeType === 1) emit("dblclick", { element: describe(el) });
    },
    true
  );

  document.addEventListener(
    "input",
    (e) => {
      const el = e.target;
      if (!el || el.nodeType !== 1) return;
      const prev = recorder.inputTimers.get(el);
      if (prev) clearTimeout(prev);
      const timer = setTimeout(() => {
        const sensitive = el.type === "password";
        emit("type", {
          element: describe(el),
          value: sensitive ? "{{credentials.password}}" : el.value,
          sensitiveInput: sensitive || undefined
        });
      }, 1000);
      recorder.inputTimers.set(el, timer);
    },
    true
  );

  document.addEventListener(
    "keydown",
    (e) => {
      if (!NONPRINTABLE_KEYS.has(e.key)) return;
      const el = e.target;
      emit("keypress", {
        key: e.key,
        element: el && el.nodeType === 1 ? describe(el) : undefined
      });
    },
    true
  );

  window.addEventListener("popstate", () => emit("navigate", {}));
}

if (recorder.sessionId) {
  startRecording();
}
