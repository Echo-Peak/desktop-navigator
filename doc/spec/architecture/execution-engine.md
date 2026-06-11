---
id: SPEC-ARCH-001
title: Execution Engine
status: draft
depends_on:
  - SPEC-PROD-001
  - SPEC-SCH-002
implements: []
---

# Goal

Define the backend architecture for the automation engine. The application
utilizes a Tauri shell (Rust core + Web UI) to parse declarative JSON schemas,
spawn local browser instances, and drive them using OS-level synthetic inputs
via the Rust `enigo` crate.

# Requirements

## 1. Overview

The backend consists of five primary modules (sections 2.1–2.5 below).

## 2. Core Architecture

### 2.1 Schema Engine

- **Responsibility:** Loads, validates, and parses the JSON schema files.
- **State Management:** Maintains a key-value store (the `StateMap`) in memory
  during execution to hold extracted variables, collections, and aggregated
  strings.

### 2.2 Browser Manager

- **Responsibility:** Spawns and manages the target browser process.
- **Enforced Determinism:** To map DOM coordinates to OS screen coordinates
  accurately, the browser MUST be spawned with strict arguments:
  - `--window-size=<width>,<height>`
  - `--window-position=<x>,<y>` (Typically `0,0`)
  - `--force-device-scale-factor=1` (Prevents OS scaling issues from breaking
    coordinates)
- **Coordinate caveat (Linux Wayland):** Absolute-coordinate clicking depends on
  knowing the window's screen position and absolute pointer positioning. This is
  reliable on X11; on Wayland it is only available through the libei/portal path
  (see section 2.4) and is therefore part of the experimental tier.

### 2.3 DOM-to-OS Bridge

- **Responsibility:** Bridges CSS selectors to absolute screen coordinates.
- **Mechanism:**
  1. Rust sends an IPC message to execute a lightweight vanilla JS snippet in
     the target browser.
  2. JS executes
     `const el = document.querySelector(selector); return el.getBoundingClientRect();`
  3. JS factors in the window position (`window.screenX`, `window.screenY`) and
     UI chrome offset (URL bar height).
  4. Returns exact `(x, y)` display coordinates back to Rust.
- **Coordinate caveat (Linux Wayland):** The bridge's absolute screen coordinates
  are only actionable on Wayland when the Input Controller can position the
  pointer via the libei/portal path (see section 2.4).

### 2.4 Input Controller (`enigo`) / UserClient

- **Responsibility:** Executes physical mouse and keyboard actions via the Rust
  `enigo` crate. Also referred to as the **UserClient**.
- **No-teleport movement invariant (hard requirement):**
  - The pointer MUST NEVER jump/teleport to a coordinate. Every move travels
    from the current cursor position to the target over a non-zero duration
    along a humanized (Bézier, variable-velocity) path. This includes the very
    first move of a session and all moves between automation steps.
  - The Input Controller maintains the current cursor position as engine state.
    Each move starts from that tracked position and ends at point X; the next
    move starts from X, and so on (A -> B, then B -> C). No action may set
    coordinates instantaneously.
  - Travel duration scales with distance (longer distance = longer travel) with
    randomized timing.
- **Humanization (behavioral bot resistance):**
  - Paths use multi-control-point Bézier curves with variable speed,
    ease-in/ease-out acceleration, and never constant-velocity straight lines.
  - Micro-behavior: small hover jitter while idle, slight target overshoot
    followed by a correction settle, randomized per-move duration.
  - Before a click or type on a scored page, perform organic idle movement, at
    least one scroll, and a randomized dwell so the session accumulates movement
    events rather than jumping straight to the target.
  - Randomized inter-keystroke delays (e.g., 50ms - 120ms) with occasional
    longer pauses.
  - These rules apply to all OS input, including captcha solving in
    [features/captcha-resolvers.md](../features/captcha-resolvers.md).
- **Linux backend strategy:**
  - X11 is the primary/default backend (enigo default `x11rb`), fully supported.
  - Wayland is best-effort and experimental: use enigo's `libei` feature via the
    XDG `RemoteDesktop` portal (GNOME 46+/KDE). This triggers a one-time
    permission prompt the user must accept.
  - Compositors without libei (e.g. some wlroots setups) are unsupported in v1;
    detect the session (`XDG_SESSION_TYPE`/`WAYLAND_DISPLAY`) and surface a
    clear error/fallback message.

### 2.5 Secret Vault

- **Responsibility:** Manages secure credential injection.
- **Mechanism:** Uses native OS keystores (macOS Keychain, Windows Credential
  Manager, Linux Secret Service) via a Rust crate like `keyring`. Resolves
  `env:SECRET_KEY` references in the schema at runtime without exposing them to
  disk or the frontend UI.

### 2.6 Action Dispatcher

- **Responsibility:** Executes the canonical action set defined in
  [schemas/action-catalog.md](../schemas/action-catalog.md). For each
  `AutomationStep`, the dispatcher reads `action.type` and routes it to its
  executor subsystem via a single registry:
  - Browser Commands (`navigate`, `reload`, `goBack`, `goForward`) -> Browser
    Manager (2.2)
  - OS Commands (`moveMouse`, `click`, `doubleClick`, `rightClick`, `type`,
    `keyboardShortcut`, `scroll`) -> DOM-to-OS Bridge (2.3) for coordinates,
    then Input Controller (2.4)
  - DOM Interactions / Data (`waitFor`, `extract`, `extractCollection`,
    `queryProperty`, `aggregateStrings`) -> DOM-to-OS Bridge (2.3) and Schema
    Engine (2.1)
- **Single point of extension:** adding a new action is done in the catalog and
  the registry only; no other module enumerates action types.
- **Execution order:** the engine reads `steps[]` in array order. It **ignores
  the presentation-only `canvas` object** (node positions, edges, viewport) in
  `PageContext` — diagram layout never affects execution.

## 3. Execution Flow

1. **Initialization:** User triggers a schema via the Tauri UI or a Cron
   scheduler.
2. **Validation:** Schema Engine validates the JSON structure and checks the
   Secret Vault for required `isSecret` keys.
3. **Boot:** Browser Manager launches the target browser at exact bounds.
4. **Step Iteration:** Engine iterates through the `PageSchema.steps` array:
   - **Wait Steps:** Pauses thread or polls DOM for visibility.
   - **Action Steps (Click/Type):** DOM-to-OS Bridge calculates coordinates ->
     Input Controller moves mouse/types OS events.
   - **Data Steps (Extract/Aggregate):** DOM extracts text/attributes -> stores
     in memory -> Aggregator compiles them.
5. **Integration Dispatch:** Resolves payload templates using the `StateMap`
   and dispatches HTTP requests (e.g., Slack Webhook).
6. **Teardown:** Browser process is terminated, and memory state is cleared.

## Acceptance

- [ ] Schema Engine loads and validates PageContext JSON
- [ ] Browser Manager spawns Chromium with enforced window size, position, and
      `--force-device-scale-factor=1`
- [ ] DOM-to-OS Bridge returns absolute screen coordinates from selectors
- [ ] Input Controller uses `enigo` with no-teleport Bézier movement and
      tracked cursor position
- [ ] Secret Vault resolves `env:SECRET_KEY` via OS keychain without plaintext
      on disk
- [ ] End-to-end step iteration runs navigate, click, type, extract, and wait
      actions
- [ ] All catalog actions dispatch to the correct subsystem via the registry
- [ ] Engine reads `steps[]` in order and ignores the `canvas` layout object
- [ ] `cargo test` passes for execution engine modules
