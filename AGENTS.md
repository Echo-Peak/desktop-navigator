# Agent instructions

## Spec-driven development

- **Entry point:** [doc/spec/README.md](doc/spec/README.md)
- Read the manifest and target spec before writing code.
- Only implement specs with `status: approved` in frontmatter.
- Do not re-implement specs with `status: implemented` unless the user asks for
  a change.

## On completion

When a spec implementation is finished and its `## Acceptance` criteria pass:

1. Update the spec file frontmatter: `status: implemented`, `implements: [...]`
2. Update the matching row in [doc/spec/README.md](doc/spec/README.md) (Status and
   Implements columns)

Do not mark a spec `implemented` if acceptance criteria are unmet or relevant
tests fail.

## New features

Create a new spec file under `doc/spec/`, add a manifest row with `status: draft`,
refine until `approved`, then implement.
