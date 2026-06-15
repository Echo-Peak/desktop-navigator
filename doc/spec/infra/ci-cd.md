---
id: SPEC-INFRA-001
title: CI/CD
status: implemented
depends_on:
  - SPEC-PROD-001
implements:
  - .github/workflows/release.yml
  - scripts/ci/set-version.mjs
  - scripts/ci/package-macos.mjs
  - scripts/ci/manifest.mjs
  - scripts/ci/cloudflare/
  - package.json
  - README.md
---

# Goal

Create a CI/CD pipeline to build and deploy the app for different platforms and
installers.

# Resolved Questions

- A Cloudflare worker fronts the private R2 bucket and resolves artifacts by
  `artifact` name (+ optional `version`) and `env`, matching the AutoUpdate
  contract (SPEC-INFRA-002). For latest downloads it also accepts a platform
  `key` (e.g. `linux-deb`), reads the env-root `manifest.json`, and redirects to
  the newest versioned object. README links use the `key` form.

# Requirements

- CI must build: macOS PKG, Windows NSIS, Windows MSI, macOS ZIP (app bundle),
  linux DEB, linux RPM. Each installer is one npm script.
- GitHub Actions is the CI/CD runner. Split each installer build into its own
  isolated step so one failure does not fail the whole job.
  - Upon installer built, add it to the current version in R2.
- Two environments: production (`main` branch) and development (`develop`
  branch).
- When a new PR is pushed to develop/main, create a new version at the workflow
  root so all jobs can reference it. Retrieve the correct version per
  environment via GH Actions caching. Default to `1.0.0` if none exists.
- R2 bucket: `desktop-navigator` (private). Path:
  `<environment>/<version>/<artifactName>.<ext>`
- Create necessary env var/secret names to push to R2 in the GH Actions workflow.
- Create `scripts/ci/cloudflare` in the project root. Add a Cloudflare worker
  script to fetch the latest artifact from the bucket (takes artifact name and
  environment).

## Acceptance

- [x] GitHub Actions builds all six installers (macOS PKG/ZIP, Windows NSIS/MSI,
      linux DEB/RPM) in isolated jobs
- [x] Artifacts upload to R2 at `<environment>/<version>/<artifactName>.<ext>`
- [x] develop and main branches map to development and production environments
- [x] Version resolved per environment via GH Actions cache (default `1.0.0`)
- [x] Cloudflare worker in `scripts/ci/cloudflare/` returns latest artifact by
      name and environment
