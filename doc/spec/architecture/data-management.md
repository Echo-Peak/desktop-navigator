---
id: SPEC-ARCH-002
title: Data Management
status: draft
depends_on:
  - SPEC-PROD-001
implements: []
---

# Goal

Define how data will be managed by the app: install paths, logs, user schemas,
captcha resolvers, and auto-update layout.

# Open Questions

- What is the ideal location for most Linux distros for userspace apps?

# Requirements

## InstallDir

Since this app will be multi-platform and will not require admin privileges, the
user dir can be used.

- Windows: `%AppData%/Local/<appname>`
- macOS: `~/Library/Application Support/<appname>`
- Linux: depends on distro (TBD)

Call this the **InstallDir**.

### AutoUpdater

The AU process is split into 2 parts: the AU module checks for updates; the
helper downloads and verifies.

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
- AU process fetches latest version (manifest.json) from GH releases via latest
  tag
- If latest version is newer than current version, continue
- Download checksum file (sha256Checksum.txt) from GH releases via latest tag
- Download ZIP file from GH releases via latest tag
- Verify ZIP file hash with sha256Checksum.txt
- If checksum matches, continue
- Unzip contents of ZIP to `v<version>` in the BrowserNavigatorUpdates directory
- Copy ZIP contents of the new version to BrowserNavigator dir. Do not delete
  previous version
- Update shortcut to point to the latest version

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
