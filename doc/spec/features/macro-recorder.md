---
id: SPEC-FEAT-003
title: Macro Recorder
status: implemented
depends_on:
  - SPEC-FEAT-001
  - SPEC-COMP-001
  - SPEC-SCH-001
  - SPEC-SCH-002
implements:
  - BrowserExtensions/Chrome/content.js
  - BrowserExtensions/Chrome/background.js
  - src-tauri/src/bridge.rs
  - src-tauri/src/lib.rs
  - src/lib/recorder.ts
  - src/routes/Automations.tsx
---

# Goal

For the /automations page, the default view, create the functionality of a
"Macro Recorder". This is for non-technical people who just wants to press
"Record" and interacts with a website with the expectation that the app will
record the users mouse, keyboard activity.

The Macro Recorder lets a user generate `PageSchemaSteps` by simply performing
actions in a real browser rather than writing JSON by hand. The user clicks
**Record** on a given PageSchema in the desktop app, a managed browser window
opens at the schema's `baseUrl`, and every click or keyboard input the user
makes is captured by the extension and streamed back to the desktop app in real
time. When the user stops recording, the captured event history can be reviewed,
edited, and exported as the schema's `steps` array.

# Unknowns / needs planning

- field managment (email, password). These need to be recorded and saved but it
  needs to be saved in order to re-play the automation

## high-level requirements

- Let non-technical users author `PageSchemaSteps` without writing JSON
- Capture enough DOM context per event that the exported step is immediately
  runnable (correct selector, correct bounding rect)
- Stream events to the desktop UI in real time so the user can see what is being
  recorded as it happens
- Password / credential inputs must never appear in plaintext in the event log
  or exported schema

## Non-Goals

- Recording network requests or API calls
- Recording inside iframes (v1 — deferred)
- Full video or screen recording
- Editing recorded events inline in v1 (delete only)

---

## User Flow

```
1. User opens a PageSchema in the desktop app
2. User clicks [Start Recording] in the schema detail view
3. Desktop app:
     a. Starts the local WebSocket bridge server (port 35282, the shared
        bridge from [components/browser-extension.md](../components/browser-extension.md))
     b. Spawns a Chrome window via the managed profile + extension
     c. Navigates to pageSchema.baseUrl
4. Browser window opens; extension activates in recording mode
5. User performs actions in the browser (clicks, typing, navigation)
6. Each action streams to the desktop app and appears as an event card
7. User clicks [Stop Recording] in the desktop app OR closes the browser
8. Recording session ends; event list is shown in full
9. User reviews the list — can delete individual events
10. User clicks [Save as Steps] → events are converted to PageSchemaSteps
    and written into the PageSchema file
```

---

## Architecture

```
┌─────────────────────────────────────────┐
│          Tauri Desktop App              │
│                                         │
│  ┌─────────────┐   ┌─────────────────┐  │
│  │ Tauri Front │◄──│  Tauri Backend  │  │
│  │  (UI/View)  │   │   (Rust)        │  │
│  └─────────────┘   └────────┬────────┘  │
│                             │           │
│                    WebSocket Server     │
│                    ws://localhost:35282 │
└─────────────────────────────────────────┘
                             ▲
                             │ ws://localhost:35282
                             │
┌────────────────────────────┴────────────┐
│         Chrome (managed profile)        │
│                                         │
│  ┌──────────────────────────────────┐   │
│  │  Extension — background.js       │   │
│  │  (service worker)                │   │
│  │  Relays events over WebSocket    │   │
│  └──────────────┬───────────────────┘   │
│                 │ chrome.runtime.sendMessage
│  ┌──────────────▼───────────────────┐   │
│  │  Extension — content.js          │   │
│  │  (injected into every page)      │   │
│  │  Listens: click, input, keydown  │   │
│  │  navigate; serialises DOM events │   │
│  └──────────────────────────────────┘   │
└─────────────────────────────────────────┘
```

### Communication flow per event

```
DOM event fires in page
  → content.js serialises → RecordedEvent
  → chrome.runtime.sendMessage(event)
  → background.js receives
  → WebSocket.send(JSON.stringify(event))
  → Tauri Rust backend receives
  → tauri::emit("recorder:event", event)
  → Tauri frontend appends event card to the Recording View
```

---

## Data Structures

### `RecordedEvent`

```typescript
interface RecordedEvent {
  id: string; // UUIDv4, unique per event
  sequence: number; // 1-indexed order in the session
  sessionId: string; // ties event to a recording session
  timestamp: number; // ms elapsed since recording started
  type: RecordedEventType;
  pageUrl: string; // full URL at time of event
  element?: ElementDescriptor; // absent for "navigate" events
  value?: string; // for "type" events: final typed value
  // ALWAYS "{{credentials.[field]}}" for
  // password inputs — never plaintext
  key?: string; // for "keypress" events: e.g. "Enter", "Tab"
}
```

### `RecordedEventType`

```typescript
type RecordedEventType =
  | "click" // any element click
  | "dblclick"
  | "type" // debounced final value of a text input
  | "keypress" // a non-character key: Enter, Tab, Escape, F-keys
  | "navigate" // page URL changed (user clicked a link / form submitted)
  | "focus"; // used for ordering context only, not exported as a step
```

### `ElementDescriptor`

```typescript
interface ElementDescriptor {
  tagName: string; // "BUTTON", "INPUT", "A", "DIV", etc.
  id?: string; // raw id attribute value
  classList: string[]; // all class names
  name?: string; // form element name attribute
  type?: string; // input type attribute
  placeholder?: string; // input placeholder
  ariaLabel?: string; // aria-label attribute
  text?: string; // innerText, max 80 chars, trimmed
  href?: string; // anchor href
  selector: string; // best auto-generated unique CSS selector
  xpath: string; // XPath fallback
  boundingRect: {
    x: number;
    y: number;
    width: number;
    height: number;
    top: number;
    right: number;
    bottom: number;
    left: number;
  };
}
```

---

## Event Capture Rules (content.js)

### Click events

- Attach a **single** `click` listener at the document root (event delegation)
- On each click: serialise the `event.target` into `ElementDescriptor`
- Capture `event.clientX` / `event.clientY` as coordinates

### Text input events

- Listen on `input` events, debounced **1 000 ms**
- After debounce fires: emit one `"type"` event with the current `field.value`
- If `field.type === "password"`: set `value` to the string literal
  `"{{credentials.password}}"` and set a flag `sensitiveInput: true`
- Never emit individual `keydown` events for printable characters

### Keypress events (non-printable keys)

- Listen on `keydown`
- Only emit for: `Enter`, `Tab`, `Escape`, `ArrowUp`, `ArrowDown`, `ArrowLeft`,
  `ArrowRight`, and F1–F12
- Printable character keys are handled by the `"type"` flow above

### Navigation events

- Listen on `window.popstate` and intercept `beforeunload`
- Also detect URL changes in the background worker via `chrome.tabs.onUpdated`
- Emit a `"navigate"` event with the new `pageUrl`

### Scroll events

- Scroll is **not** captured as a recorded step. On replay the executor
  auto-scrolls the target into view (`scrollIntoView`) before each action, so
  explicit scroll steps are unnecessary.

---

## CSS Selector Generation

The content script must generate a CSS selector that is unique on the page.
Priority order:

1. `#id` — if the element has a non-empty, unique `id`
2. `tagName[name="value"]` — for form elements with a `name` attribute
3. `tagName.class1.class2` — if the combination is unique on the page
4. `tagName[aria-label="..."]` — if aria-label is present
5. `tagName[placeholder="..."]` — for inputs with placeholder
6. Full ancestor path — `div.container > ul > li:nth-child(2) > button`

Always verify uniqueness with
`document.querySelectorAll(candidate).length === 1`. If no unique selector can
be built, fall back to XPath.

---

## WebSocket Protocol

The Tauri backend starts the server when a recording session begins and closes
it on stop. The extension connects immediately on window open.

### Message types (extension → backend)

```jsonc
// Event captured
{ "msg": "event", "payload": RecordedEvent }

// Extension signals it is connected and ready
{ "msg": "ready", "sessionId": "abc123" }

// Extension signals browser window was closed by user
{ "msg": "browser_closed" }
```

### Message types (backend → extension)

```jsonc
// Backend tells extension to begin capturing
{ "msg": "start", "sessionId": "abc123" }

// Backend tells extension to pause (user clicked Pause in desktop app)
{ "msg": "pause" }

// Backend tells extension to stop
{ "msg": "stop" }
```

### Session token

The Tauri backend generates a random `sessionId` (UUIDv4) per recording session.
The token is passed as a query parameter on the URL Chrome navigates to on launch
(`baseUrl?_bpsession=abc123`). The content script reads it from
`window.location.search` and forwards it to the background worker via
`chrome.runtime.sendMessage`; the background worker includes it in the `"ready"`
message. The backend rejects WebSocket messages with a mismatched or absent
`sessionId`. The content script strips `_bpsession` from any captured/recorded
URLs so it never leaks into exported steps.

---

## Desktop App — Recording View

The Recording View replaces the normal PageSchema detail view when a session is
active.

### Header bar

| Element      | Detail                                                  |
| ------------ | ------------------------------------------------------- |
| Status badge | `● RECORDING` (red, pulsing) / `⏸ PAUSED` / `■ STOPPED` |
| Timer        | Elapsed time `mm:ss.ms`, counting up                    |
| Event count  | `12 events`                                             |
| Pause button | Sends `pause` to extension; timer pauses                |
| Stop button  | Sends `stop`; session ends; Save panel appears          |

### Node-graph canvas

The Recording View uses the **same React Flow (`@xyflow/react`) canvas** as the
Manual tab (see [features/desktop-ui.md](desktop-ui.md)). There is no separate
event-card feed.

- Each incoming `RecordedEvent` is converted to a catalog `AutomationAction`
  (see Export mapping below) and appended to the canvas as a node.
- Each new node is connected with an arrow from the previously added node
  (A -> B -> C), so the diagram grows chronologically as the user acts.
- Nodes are auto-positioned as they arrive (e.g. a vertical chain); positions
  become editable after recording stops.
- The canvas auto-pans to keep the newest node in view.
- A node still surfaces its event detail (action label, selector/element text,
  bounding rect, `🔒 Sensitive` badge for password fields) within the node body.

```
[Navigate https://app.example.com]
        │
        ▼
[Type  Input[name="email"]  user@example.com]
        │
        ▼
[Type  Input[type=password]  🔒 {{credentials.password}}]
        │
        ▼
[Click  Button#submitBtn  "Sign In"]
```

### Save panel (shown after stop)

- **[Save as Steps]** — converts events and writes to `PageSchema.steps`
  (overwrites existing steps after a confirmation dialog). Also writes the
  `canvas` object (node positions + edges) so the recorded automation re-renders
  with its layout; auto-layout positions are persisted but user-editable.
- **[Append to Steps]** — appends after any existing steps (and appends matching
  `canvas` nodes/edges)
- **[Discard]** — throws away the session

---

## Sensitive Input Handling

When a `"type"` event arrives with `sensitiveInput: true`:

1. The event card shows a 🔒 badge and the value displays as
   `{{credentials.password}}` — never the real value
2. After the session ends, if any sensitive events exist, the desktop app shows
   a prompt: _"We detected a password field. Save this credential to your
   keychain so the schema can use it securely."_ with an input for the user to
   type (or confirm) the password into the OS keychain
3. The exported step's `params.value` is `"{{credentials.password}}"` — the
   keychain reference, never plaintext

---

## Export: RecordedEvent → AutomationAction

`RecordedEventType` is the capture-side vocabulary. The authored/executed
vocabulary is the canonical `AutomationAction` union in
[schemas/action-catalog.md](../schemas/action-catalog.md). This table is the only
bridge between the two.

| `RecordedEvent.type` | Catalog `AutomationAction`                                              |
| -------------------- | ----------------------------------------------------------------------- |
| `click`              | `{ type: "click", selector: element.selector }`                         |
| `dblclick`           | `{ type: "doubleClick", selector: element.selector }`                   |
| `type`               | `{ type: "type", selector: element.selector, value: event.value }`      |
| `keypress`           | `{ type: "keyboardShortcut", keys: [event.key] }` (single-key)          |
| `navigate`           | `{ type: "navigate", url: event.pageUrl }`                              |

`scroll` and `focus` events are not exported. Scrolling is handled by the
executor auto-scrolling the target into view before each action; `focus` is used
only for ordering context (Tab navigation exports a single `keypress: Tab`).

Each exported step also gets:

- `description` auto-populated from `element.text` or `element.ariaLabel` if
  available (e.g., `"Click 'Sign In' button"`)
- `timeoutMs: 10000` (page default)
- A `waitFor` step (catalog action with `condition: { type: "elementVisible" }`)
  is **prepended** before each `click`/`doubleClick`/`type` step to make replays
  resilient:

```jsonc
// Prepended automatically
{ "id": "wait_submit", "action": { "type": "waitFor", "condition": { "type": "elementVisible", "selector": "button#submitBtn" } }, "timeoutMs": 10000 },
// Then the actual action
{ "id": "click_submit", "action": { "type": "click", "selector": "button#submitBtn" }, "description": "Click 'Sign In' button" }
```

---

## Replay

A recorded and saved session can be replayed via the normal PageSchema step
runner. No special replay mode is needed — the exported steps are standard
`PageSchemaSteps` and run through the existing CDP + enigo execution pipeline.

The **Record** button in the schema detail view shows a secondary
**[Re-record]** label when steps already exist, and warns the user that saving
will overwrite the current steps.

---

## Edge Cases

| Scenario                                                | Handling                                                                                                                                                             |
| ------------------------------------------------------- | -------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| User clicks a dynamically rendered element (React, SPA) | Selector generation runs at click-time against the live DOM, so the element exists. The prepended `waitForElement` on replay handles timing.                         |
| Element has no unique selector                          | Fall back to XPath; flag the event card with a ⚠ warning in the UI                                                                                                   |
| User closes the browser window without clicking Stop    | Extension emits `"browser_closed"` over WebSocket; backend treats this as an implicit stop                                                                           |
| WebSocket connection drops mid-session                  | Extension queues events locally (in-memory array) and attempts reconnect every 2 s; flushes queue on reconnect                                                       |
| User navigates away mid-recording (multi-page flow)     | Content script is re-injected on each page load via `content_scripts: [{ matches: ["<all_urls>"] }]` in the manifest; navigation is captured as a `"navigate"` event |
| Two recording sessions start at the same time           | Backend enforces one active session per app instance; second [Start Recording] is disabled while a session is live                                                   |

---

## Resolved decisions

- **Token passing** — passed as a `_bpsession` URL query param on launch and read
  by the content script (see ### Session token).
- **Scroll step export** — scroll is not exported; the executor auto-scrolls the
  target into view before each action.
- **`focus` events** — captured for ordering context only; Tab navigation exports
  a single `keypress: Tab` step (no separate `focus` step).
- **Multi-step type grouping** — only the final debounced value of a field is
  captured (intended behaviour).

## Open Questions

1. **Iframe support** — content script is not injected into cross-origin iframes
   by default. Deferred to v2; worth calling out in the UI with a notice when
   the user clicks inside an iframe.

2. **Re-record confirmation UX** — if a schema already has hand-written steps,
   the warning dialog needs to be explicit that re-recording overwrites them.
   Consider an **Export to new file** option instead.

## Acceptance

- [ ] Record button opens managed Chrome with extension at `pageSchema.baseUrl`
- [ ] Events stream to desktop UI in real time via WebSocket
- [ ] Click, type, keypress, scroll, and navigate events captured per rules in
      this spec
- [ ] Password fields never stored in plaintext; exported as credential refs
- [ ] Events render as nodes on the shared React Flow canvas, appended with an
      arrow from the previous node in chronological order
- [ ] Stop recording shows Save panel; Save as Steps writes valid PageContext
      `steps[]` and a `canvas` object (positions + edges)
- [ ] Each `RecordedEvent.type` maps to a catalog `AutomationAction` per the
      Export table; no orphan action names
- [ ] Exported steps include a prepended `waitFor` (`elementVisible`) before
      click/doubleClick/type
- [ ] One active recording session enforced per app instance
