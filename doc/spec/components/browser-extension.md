---
id: SPEC-COMP-001
title: Browser Extension
status: draft
depends_on:
  - SPEC-ARCH-001
implements: []
---

# Goal

Create a Chrome extension that bridges the desktop app and Rust backend to the
DOM. The extension executes read/query actions upon the page:

- Detecting page title
- Detecting element coordinates / `getBoundingClientRect`
- Scrolling to a specific location on the page
- Getting page/document details such as total height/width

Captcha detection and macro recording also depend on this component.

For v1, focus only on Chrome. The extension runs in an isolated browser window
with an isolated user profile directory.

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

## Tab and window management

Window/tab management is required so the backend/UserClient can switch between
tabs (page contexts). Each pageContext/tab/page gets an `id` for tracking and
automation.

## Acceptance

- [ ] Extension lives at `BrowserExtensions/Chrome/`
- [ ] WebSocket connects to desktop app on `localhost:35282` with port retry
      (35282–35292, 1000ms delay)
- [ ] DOM queries work: page title, `getBoundingClientRect`, scroll, page
      dimensions
- [ ] Extension runs headless (no popup UI) in isolated Chrome profile
- [ ] Tab/window IDs exposed for backend tab switching
