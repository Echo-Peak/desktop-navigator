---
id: SPEC-COMP-001
title: Browser Extension
status: implemented
depends_on:
  - SPEC-ARCH-001
implements:
  - BrowserExtensions/Chrome/
  - src-tauri/src/adapters/browser.rs
---

> Implementation note: the MV3 extension is implemented in
> `BrowserExtensions/Chrome/` (headless background service worker WS client with
> 35282–35292 port retry, DOM content script for title/rect+screen-coords/
> text/attribute/scroll/page-dimensions/collection, and tab/window management).
> Port-retry logic is unit-tested (`pnpm test:ext`). Live DOM/IPC behaviour
> requires loading the unpacked extension in Chrome to verify end-to-end.

# Goal

Create a Chrome extension that bridges the desktop app and Rust backend to the
DOM. The extension executes read/query actions upon the page:

- Detecting page title
- Detecting element coordinates / `getBoundingClientRect`
- Scrolling to a specific location on the page
- Getting page/document details such as total height/width

Captcha detection and macro recording also depend on this component.

For v1, focus only on **Chromium-based browsers** (Chromium, Chrome for
Testing). Branded Google Chrome is out of scope for v1 (see ### Packaging and
loading). The extension runs in an isolated browser window with an isolated user
profile directory.

# Requirements

## IPC bridge

- WebSocket connection to the desktop app with retry logic.
- No extension UI; headless bridge only.
- Project path: `BrowserExtensions/Chrome` at the project root.
- Default WS client settings:
  ```
  PORT=35282
  HOST=localhost
  ```
- Increment port on connection failure, up to +10 ports (35282–35292), 1000ms
  delay between attempts.

## Packaging and loading

The extension ships **unpacked** (plain `manifest.json` + JS) as a Tauri bundle
resource (`bundle.resources` in `tauri.conf.json` → `BrowserExtensions/Chrome`).
It is therefore copied into every installer (deb/rpm/nsis/msi/app/dmg) and
resolved at runtime via `resource_dir()`. Because it lives inside the versioned
app directory, the AutoUpdater (SPEC-INFRA-002) ships extension updates together
with the app — no separate extension update channel.

The app never touches the user's normal browser. It launches a **managed**
Chromium with an isolated `--user-data-dir` (under `InstallDir/profile`) and a
`--remote-debugging-port`.

Loading strategy (v1, Chromium-only):

- **Chromium / Chrome for Testing:** loaded via `--load-extension` +
  `--disable-extensions-except` into the isolated profile. This is the supported
  v1 path (`src-tauri/src/adapters/browser.rs`).

Deferred (branded Google Chrome support, post-v1):

- Branded Chrome 137+ ignores `--load-extension`. Supporting it requires either
  the CDP `Extensions.loadUnpacked` command — which is only available over
  `--remote-debugging-pipe` (an fd-based pipe, not the WebSocket port) together
  with `--enable-unsafe-extension-debugging` — or an `ExtensionSettings` /
  `ExtensionInstallForcelist` enterprise policy with a fixed extension id
  (manifest `key`) installed per-OS by the installer.

### Open question

- When branded-Chrome support is added: pipe-based CDP `loadUnpacked` vs.
  enterprise policy, and whether to bundle Chrome for Testing (~150MB) to
  guarantee a known browser.

## Tab and window management

Window/tab management is required so the backend/UserClient can switch between
tabs (page contexts). Each pageContext/tab/page gets an `id` for tracking and
automation.

## Acceptance

- [x] Extension lives at `BrowserExtensions/Chrome/`
- [x] WebSocket connects to desktop app on `localhost:35282` with port retry
      (35282–35292, 1000ms delay)
- [x] DOM queries work: page title, `getBoundingClientRect`, scroll, page
      dimensions
- [x] Extension runs headless (no popup UI) in isolated Chrome profile
- [x] Tab/window IDs exposed for backend tab switching
