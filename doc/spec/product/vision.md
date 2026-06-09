---
id: SPEC-PROD-001
title: Product Vision
status: draft
depends_on: []
implements: []
---

# Goal

Create a desktop app using Tauri to control the system/OS in order to automate a
web browser.

The user will use the UI of the app as the gateway to set up "schemas" which are
JSON files stored locally.

This app will make use of OpenRouter to facilitate some operations like computer
vision and simple NLP processing.

# Open Questions

- How will the ability to use multiple tabs work? The UserClient / mouse will
  need to somehow locate tab 1 in the newly created window. An algorithm will
  need to take into account window DPI, resolution, bookmark bar, tab bar
  orientation. It makes sense to use keyboard input/shortcuts rather than mouse.
  In most cases, opening a new browser window will automatically focus on the
  address bar. In Chromium browsers `ctrl+<number row>` changes tab.

# Requirements

## High-level

- Rust based backend to simulate keyboard and mouse input. Mouse inputs needed
  are movement, scroll wheel and left click. See
  [architecture/execution-engine.md](../architecture/execution-engine.md) section
  2.4 (UserClient / Input Controller).
- Schema system to orchestrate the app. See
  [schemas/page-context.md](../schemas/page-context.md).
- Frontend / UI. See [features/desktop-ui.md](../features/desktop-ui.md).
- CI/CD pipeline to build and deploy installers to Cloudflare R2. See
  [infra/ci-cd.md](../infra/ci-cd.md).
- OS-level credential enclave for environment variables used when logging into
  pages.
- When the app is active and has a browser associated, render a full-screen
  overlay with a translucent background (0.6 opacity) without affecting
  mouse/keyboard. Show a UI control bar at the bottom center with actions
  triggered via keyboard. Use only the escape key to terminate the UserClient
  session.
- Chrome extension as the IPC bridge between the DOM and the Tauri/Rust backend.
  See [components/browser-extension.md](../components/browser-extension.md).
- AutoUpdate module for the desktop app (not the extension). See
  [architecture/data-management.md](../architecture/data-management.md) (###
  AutoUpdater).
- App config file for WS server port and host. Defaults:
  ```
  PORT=35282
  HOST=localhost
  ```
- Increment port on connection failure, up to +10 ports (35282–35292), 1000ms
  delay between attempts.
- LLM module/API for OpenRouter: async, event-driven; frontend sends images and
  text and receives responses.
- File logger: write to FS when compiled, otherwise console/stdout. See
  [architecture/data-management.md](../architecture/data-management.md) (### Logs
  dir).
- Captcha resolver module for user-created resolvers. See
  [features/captcha-resolvers.md](../features/captcha-resolvers.md).

## Architecture

- A **cycle** is a UI start/stop unit that can run multiple page automations in
  sequence.
- The start/stop of a cycle is a **session** in which the app is actively
  operating on a browser on the host.

## Acceptance

Parent spec — not implemented directly. Mark `implemented` when all dependent
child specs in [README.md](../README.md) are `implemented`.
