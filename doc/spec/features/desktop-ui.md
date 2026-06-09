---
id: SPEC-FEAT-001
title: Desktop UI
status: draft
depends_on:
  - SPEC-PROD-001
  - SPEC-SCH-001
implements: []
---

# Goal

Create an interactive UI/UX with transitions/animations using React Motion for
the Tauri app frontend. The frontend creates browser/page automations visually
and via JSON (two editor modes).

The UI/UX should be user friendly. The app targets primarily non-technical
people.

**Tech stack:** React, Vite, shadcn/shadcn-react, React Motion, React Router.

# Requirements

## High-level overview

- Demo/guide feature upon app start.
- Global UI control bar near the top to start browser automation.
- Global UI sidebar: "Browser config", "Automations", "Integrations", "Tasks".
- Component using the browser extension IPC to stream user actions (see
  [macro-recorder.md](macro-recorder.md)).

## Sidebar

- Browser config → `/browser-config`
- Automations → `/automations`
- Integrations → `/integrations`
- Tasks → `/tasks`
- LLM helpers → `/llm-helpers`
- Captcha Resolvers → `/captcha-resolvers`

### Pages / routes

#### Homepage, `/`

Grid of automation PageContext/schema cards (4 columns). Each card shows domain
name, favicon (fetched on first create), and description.

#### Browser config, `/browser-config`

Browser settings: executable path, flags, maximized mode, etc.

#### Automations, `/automations`

Manage all automations/pages.

#### Integrations, `/integrations`

Create integrations for automation output.

#### Tasks, `/tasks`

Arrange automations: sequence, repeat counts, cycles, triggers. Triggers v1:
start cycle on cursor idle duration; stop cycle on right-click or escape.

Users can group page automations into named **cycles**. The global control bar
shows a cycle dropdown; play runs only the selected cycle. Default cycle
"Default" runs all page automations with no grouping.

#### Captcha Resolvers, `/captcha-resolvers`

List built-in and user-created captcha resolvers as cards showing
`resolver.title`. See
[features/captcha-resolvers.md](captcha-resolvers.md) for types/interfaces.

## User actions streaming

Component working with the browser extension to record click/keyboard history on
DOM elements (macro streaming/replaying). User clicks record for a pageContext;
app opens a browser window with the extension, navigates to the domain, and
records mouse/keyboard events. The desktop view shows entries like
`Button#someButton` with `clientBoundingRect` details.

## Acceptance

- [ ] All routes in this spec render (`/`, `/browser-config`, `/automations`,
      `/integrations`, `/tasks`, `/llm-helpers`, `/captcha-resolvers`)
- [ ] Global control bar and sidebar navigation work
- [ ] Automation cards show domain, favicon, and description on homepage
- [ ] Cycle dropdown in control bar runs selected automation group
- [ ] Session overlay dims screen (0.6 opacity) without blocking input; Escape
      terminates UserClient
- [ ] Demo/guide shown on first app start
