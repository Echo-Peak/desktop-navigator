---
id: SPEC-ARCH-002
title: Data Management
status: approved
depends_on:
  - SPEC-PROD-001
implements: []
---

# Goal

Define how data will be managed by the app: install paths, logs, user schemas,
captcha resolvers, and auto-update layout.

# Requirements

## InstallDir

Since this app will be multi-platform and will not require admin privileges, the
user dir can be used.

- Windows: `%AppData%/Local/<appname>`
- macOS: `~/Library/Application Support/<appname>`
- Linux: `$XDG_DATA_HOME/<appname>`, defaulting to `~/.local/share/<appname>`
  when `XDG_DATA_HOME` is unset

Call this the **InstallDir**.

### AutoUpdater

The AU process is split into 2 parts: the AU module checks for updates; the
helper downloads and verifies. Full behaviour is specified in
[infra/autoupdate.md](../infra/autoupdate.md) (SPEC-INFRA-002); this section only
defines the on-disk layout.

Artifacts are sourced from Cloudflare R2 via the worker defined in
[infra/ci-cd.md](../infra/ci-cd.md) (SPEC-INFRA-001), not GitHub Releases.

Example layout (Windows):

```
.
└── %AppData%/
    └── local/
        ├── BrowserNavigator/
        │   ├── v0.9.2-dev.12
        │   ├── v1.0.0-release.0
        │   └── shortcutLauncher
        └── BrowserNavigatorUpdates/
            └── v0.9.2-dev.13
```

AU flow:

- AU process starts
- AU process gets current app version
- AU process fetches latest version (manifest.json) from the R2 worker for the
  active environment
- If latest version is newer than current version, continue
- Download checksum file (sha256Checksum.txt) from R2
- Download ZIP file from R2
- Verify ZIP file hash with sha256Checksum.txt
- If checksum matches, continue
- Unzip contents of ZIP to `v<version>` in the BrowserNavigatorUpdates directory
- Copy ZIP contents of the new version to BrowserNavigator dir. Do not delete
  previous version (all prior versions are retained)
- Update shortcut to point to the latest version
- Prompt the user to restart now or later

Additional AU behavior (from product vision): run on app start and every 2 hours;
if a cycle is active, wait until idle. Use a CLI `--update` flag via a helper
copy of the app in a temp directory.

### Logs dir

Store logs within InstallDir as the folder `logs`. Filename:
`<timestamp>-<version>.log`

### User data (schemas) dir

Schema files: `InstallDir/UserData/Pages`

### Captcha resolvers dir

User-created captcha resolvers: `InstallDir/<appVersion>/captcha-resolvers`

## Acceptance

- [ ] InstallDir resolves correctly on Windows, macOS, and Linux
- [ ] Logs written to `InstallDir/logs/<timestamp>-<version>.log`
- [ ] Page schemas stored under `InstallDir/UserData/Pages`
- [ ] Captcha resolvers stored under `InstallDir/<appVersion>/captcha-resolvers`
- [ ] AutoUpdater checks GH releases, verifies SHA-256, and installs to versioned
      directories without deleting previous versions
