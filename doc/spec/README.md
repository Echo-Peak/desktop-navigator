# Specification Index

Normative specs for desktop-navigator. Each spec has a stable `id`, `status`,
`depends_on`, and `implements` list in YAML frontmatter.

| ID | Spec | Status | Implements | Depends on |
| --- | --- | --- | --- | --- |
| SPEC-PROD-001 | [product/vision.md](product/vision.md) | draft | — | — |
| SPEC-ARCH-001 | [architecture/execution-engine.md](architecture/execution-engine.md) | draft | — | SPEC-PROD-001 |
| SPEC-ARCH-002 | [architecture/data-management.md](architecture/data-management.md) | draft | — | SPEC-PROD-001 |
| SPEC-SCH-001 | [schemas/page-context.md](schemas/page-context.md) | draft | — | SPEC-ARCH-001, SPEC-ARCH-002 |
| SPEC-FEAT-001 | [features/desktop-ui.md](features/desktop-ui.md) | draft | — | SPEC-PROD-001, SPEC-SCH-001 |
| SPEC-FEAT-002 | [features/captcha-resolvers.md](features/captcha-resolvers.md) | draft | — | SPEC-ARCH-001, SPEC-COMP-001 |
| SPEC-FEAT-003 | [features/macro-recorder.md](features/macro-recorder.md) | draft | — | SPEC-FEAT-001, SPEC-COMP-001, SPEC-SCH-001 |
| SPEC-COMP-001 | [components/browser-extension.md](components/browser-extension.md) | draft | — | SPEC-ARCH-001 |
| SPEC-INFRA-001 | [infra/ci-cd.md](infra/ci-cd.md) | draft | — | SPEC-PROD-001 |

Non-normative backlog: [../backlog/post-mvp.md](../backlog/post-mvp.md)

## Agent workflow

Read this file before implementing anything in the repository.

1. Find the target spec by `id` in the table above and open its markdown file.
2. Only implement specs with `status: approved`. Do not implement `draft` specs
   unless the user explicitly approves them for implementation.
3. Do not modify specs with `status: implemented` unless the user requests a
   change or bug fix to shipped behavior.
4. Implement according to the spec's `# Requirements` and satisfy every item in
   `## Acceptance`.
5. When implementation is complete and acceptance criteria pass:
   - Set the spec frontmatter `status: implemented`.
   - Set `implements:` to the repo paths of code created or owned by this spec
     (e.g. `src-tauri/src/engine/`).
   - Update the **Status** and **Implements** columns for that row in the table
     above.
6. Do not mark `implemented` if acceptance criteria are not met or tests fail.
7. To add a new feature post-launch: create a new spec under the appropriate
   subdirectory, add a row to this table with `status: draft`, get it to
   `approved`, then implement.

## Status values

- `draft` — work in progress, not yet approved for implementation
- `review` — ready for review
- `approved` — signed off, implementation may proceed
- `implemented` — matches shipped behavior; `implements` paths must be filled in
- `superseded` — replaced by another spec; do not implement

## Spec template

Every normative spec should include:

1. YAML frontmatter (`id`, `title`, `status`, `depends_on`, `implements`)
2. `# Goal`
3. `# Requirements` (or domain-specific sections)
4. `# Open Questions` (optional)
5. `## Acceptance` — checklist or test commands that define "done"
