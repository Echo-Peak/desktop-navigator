# Desktop Navigator

Automate the web from your desktop — visually, locally, and under your control.

Desktop Navigator is a Tauri desktop app that drives Chromium-based browsers through real OS input (keyboard and mouse). You design automations in a visual editor or by recording your actions, store them as JSON on your machine, and run them as cycles whenever you need to. A bundled Chrome extension bridges the browser DOM to the Rust backend; credentials stay in your OS keychain.

Built for people who want repeatable browser workflows without shipping data to a remote automation service.

## Download

Latest production installers:

| Platform | Format | Download |
| --- | --- | --- |
| Windows | NSIS | [Download](https://desktop-navigator-artifacts.echopeakdev.workers.dev/?key=win-nsis&env=production) |
| Windows | MSI | [Download](https://desktop-navigator-artifacts.echopeakdev.workers.dev/?key=win-msi&env=production) |
| macOS | PKG | [Download](https://desktop-navigator-artifacts.echopeakdev.workers.dev/?key=mac-pkg&env=production) |
| macOS | ZIP (app bundle) | [Download](https://desktop-navigator-artifacts.echopeakdev.workers.dev/?key=mac-zip&env=production) |
| Linux | DEB | [Download](https://desktop-navigator-artifacts.echopeakdev.workers.dev/?key=linux-deb&env=production) |
| Linux | RPM | [Download](https://desktop-navigator-artifacts.echopeakdev.workers.dev/?key=linux-rpm&env=production) |

Development builds use the same links with `env=development` instead of `env=production`.

The app checks for updates automatically and prompts you before restarting.

## Features

- **Visual automation editor** — Build step-by-step workflows on a node canvas; no code required for most tasks.
- **Macro recorder** — Record clicks, typing, and navigation in the browser; convert events into reusable steps.
- **Cycles and tasks** — Group page automations into named cycles and run them from a single Play control.
- **Chromium integration** — Launches an isolated browser profile with the bundled extension pre-loaded.
- **Secure credentials** — Login secrets are stored in the OS keychain, not in plain JSON.
- **Integrations** — Send automation output to webhooks or files; optional LLM helpers via OpenRouter or local Ollama.
- **Captcha helpers** — Pluggable resolvers for common challenge patterns.

During an active session, a translucent overlay shows that automation is running. Press **Esc** to stop.

## How it works

```
┌─────────────────┐     WebSocket      ┌──────────────────┐
│  Desktop UI     │◄──────────────────►│  Chrome extension │
│  (React)        │                    │  (DOM bridge)     │
└────────┬────────┘                    └────────┬─────────┘
         │                                      │
         │ Tauri commands                       │ page queries,
         ▼                                      │ events
┌─────────────────┐                             │
│  Rust backend   │◄────────────────────────────┘
│  execution      │
│  engine         │────► OS input (keyboard, mouse)
└─────────────────┘
         │
         ▼
   Local JSON schemas, logs, and keychain secrets
```

Automations are defined as **page contexts** — JSON files that describe a domain, steps, and optional canvas layout. A **cycle** runs one or more pages in sequence inside a **session** (from Play until done or Esc).

## Requirements

- **Runtime:** A Chromium-based browser (Chromium, Chrome, or similar). Branded Chrome is supported; extension loading uses `--load-extension`.
- **Platforms:** Windows, macOS, or Linux (see download table above).

## Development

**Stack:** Tauri 2, Rust, React, Vite, TypeScript.

```bash
pnpm install
pnpm tauri dev
```

Other useful commands:

```bash
pnpm build          # frontend production build
pnpm typecheck      # TypeScript
pnpm test:ext       # extension unit tests
cargo test -p navigator-core
```

Linux dev builds need WebKit/GTK dependencies (same set as the CI workflow). See [`.github/workflows/release.yml`](.github/workflows/release.yml) for the apt package list.

## Project docs

Specifications and agent workflow live in [doc/spec/README.md](doc/spec/README.md).
