---
id: SPEC-INFRA-002
title: AutoUpdate Module
status: implemented
depends_on:
  - SPEC-PROD-001
  - SPEC-ARCH-002
  - SPEC-INFRA-001
implements:
  - crates/core/src/update/
  - crates/core/src/log.rs
  - src-tauri/src/update.rs
  - src-tauri/src/lib.rs
  - src/components/UpdatePrompt.tsx
  - src/components/Toast.tsx
---

# Goal

Implement a self-update mechanism for the desktop app (Tauri/Rust) that silently
checks for new versions, downloads and verifies the installer artifact, installs
it into a versioned directory, and updates the launch shortcut — without
interrupting an active automation cycle.

# Requirements

## Process split

The update flow is divided into two roles:

- **AU module** — runs inside the main app process; polls for updates and
  orchestrates the flow.
- **AU helper** — a second copy of the app binary placed in a temp directory;
  invoked with `--update <version>` to perform the download, verify, and install
  steps independently of the running instance.

## Trigger conditions

- Run on app start, before the main window opens.
- Run on a 2-hour repeating timer while the app is in an idle state.
- If a cycle is active, defer the check until the cycle reaches idle.

## Version manifest

Fetch `manifest.json` from the Cloudflare R2 worker defined in SPEC-INFRA-001,
scoped to the active environment (`production` or `development`):

```
GET <worker-url>?artifact=manifest.json&env=<environment>
```

> Worker contract (provisional): the request shapes below are assumed by the
> client until SPEC-INFRA-001 is implemented and must be kept in sync with the
> worker. The worker URL and environment are resolved at runtime with the
> precedence: env vars (`DN_UPDATE_WORKER_URL`, `DN_UPDATE_ENV`) > `update.json`
> in `InstallDir` > compiled defaults (env baked from the CI branch, else derived
> from the build profile). When no worker URL resolves, AU is disabled and logs a
> single line.
>
> - Checksum: `GET <worker-url>?artifact=sha256Checksum.txt&env=<environment>&version=<v>`
> - Artifact: `GET <worker-url>?artifact=<artifactName>&env=<environment>&version=<v>`

`manifest.json` schema:

```json
{
  "version": "1.2.3",
  "releaseDate": "2026-06-11T00:00:00Z",
  "artifacts": {
    "win-nsis":  "BrowserNavigator-1.2.3-setup.exe",
    "win-msi":   "BrowserNavigator-1.2.3.msi",
    "mac-pkg":   "BrowserNavigator-1.2.3.pkg",
    "mac-zip":   "BrowserNavigator-1.2.3.app.zip",
    "linux-deb": "browser-navigator_1.2.3_amd64.deb",
    "linux-rpm": "browser-navigator-1.2.3.x86_64.rpm"
  }
}
```

Compare `manifest.version` against the running app version using semver ordering.
If `manifest.version <= current`, do nothing and exit the AU flow.

## Download and verify

1. Download `sha256Checksum.txt` from the artifact store for the target version.
2. Download the platform-appropriate artifact ZIP.
3. Compute SHA-256 of the downloaded file and compare against the checksum file.
4. On mismatch: delete the downloaded file, log the failure, and abort.

## Install layout

Follow the directory layout defined in SPEC-ARCH-002 (### AutoUpdater):

```
<InstallDir>/
  v<current>/        ← running version, never deleted
  shortcutLauncher   ← symlink/shortcut pointing to the active binary
<InstallDirUpdates>/
  v<new>/            ← staging area for the newly downloaded version
```

Install steps:

1. Unzip artifact to `<InstallDirUpdates>/v<new>/`.
2. Copy the unzipped contents to `<InstallDir>/v<new>/`.
3. Rewrite `shortcutLauncher` to point to `<InstallDir>/v<new>/<binary>`.
4. Do **not** delete any previous version directory — all prior versions are
   retained indefinitely.

## Post-install restart

After a successful install, prompt the user with a "Restart now / Restart later"
choice. Never force-restart. If "later" is chosen, the new version takes effect
on the next launch via `shortcutLauncher`.

## CLI flag

The AU helper is launched as:

```
<binaryPath> --update <newVersion>
```

When this flag is detected at startup the binary must perform only the
download/verify/install sequence, write a success or error line to stdout, and
exit without opening the main window.

## Error handling

| Failure | Behaviour |
|---|---|
| Network / manifest fetch failure | Log, retry on next trigger |
| Checksum mismatch | Delete artifact, log, abort |
| Disk write permission error | Log, show toast notification in UI |
| Helper process non-zero exit | Log stderr output, surface toast |

## Logging

All AU events must be written to the file logger defined in SPEC-ARCH-002
(### Logs dir): check triggered, version comparison result, download
start/end, checksum pass/fail, install success/fail.

## Acceptance

- [ ] AU module triggers on app start
- [ ] AU module triggers every 2 hours while idle
- [ ] AU defers when a cycle is active and resumes on idle
- [ ] `manifest.json` fetched from R2 worker; update skipped when already current
- [ ] Artifact and checksum downloaded; SHA-256 verified
- [ ] Artifact deleted and flow aborted on checksum mismatch
- [ ] New version unzipped to staging, then copied to `<InstallDir>/v<new>/`
- [ ] Previous version directory preserved after install
- [ ] `shortcutLauncher` rewritten to new version binary
- [ ] `--update <version>` flag runs helper flow without opening the UI
- [ ] User prompted to restart now or later after a successful install
- [ ] All AU events written to file logger
- [ ] Toast notification shown on write permission error
