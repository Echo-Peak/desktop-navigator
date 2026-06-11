---
id: SPEC-FEAT-001
title: Desktop UI
status: draft
depends_on:
  - SPEC-PROD-001
  - SPEC-SCH-001
  - SPEC-SCH-002
implements: []
---

# Goal

Create an interactive UI/UX with transitions/animations using React Motion for
the Tauri app frontend. The frontend creates browser/page automations visually
and via JSON (two editor modes).

The UI/UX should be user friendly. The app targets primarily non-technical
people.

**Tech stack:** React, Vite, shadcn/shadcn-react, React Motion, React Router,
`@xyflow/react` (React Flow) for the node-graph / drag-and-drop canvas.

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
- LLM integrations → `/llm-integrations`
- Captcha Resolvers → `/captcha-resolvers`

### Pages / routes

#### Homepage, `/`

Grid of automation PageContext/schema cards (4 columns). Each card shows domain
name, favicon (fetched on first create), and description.

#### Browser config, `/browser-config`

Browser settings: executable path, flags, maximized mode, etc.

#### Automations, `/automations`

This is the page in which the user sets up an automation for a given url/page.
It serves two user groups, non-technical and technical. The default view is the
"Recorder" view; the optional view is the "Manual" view where the user picks
actions/commands and arranges them on a node-graph diagram to build a highly
custom automation.

Both views share **one node-graph canvas** built with `@xyflow/react` (React
Flow). Nodes are actions from the canonical catalog
([schemas/action-catalog.md](../schemas/action-catalog.md)) — a node's `data` is
the `AutomationStep`. Each node connects to the next with an arrow in
chronological order (point A, e.g. a `moveMouse`, points to point B, e.g. a
`click`, the most recently added node).

There are 2 tabs: Tab 1 "Recorder" and Tab 2 "Manual". Below the tabs is the
shared canvas content area.

- **Manual tab:** drag actions from the Expandable panel dialog onto the canvas;
  connect and reorder nodes. Node positions are user-controlled.
- **Recorder tab:** nodes are appended live during recording, each with an arrow
  from the previous node to the newly added node. Positions are auto-assigned as
  nodes arrive and remain editable after recording stops. Same canvas component
  as Manual.

By "automation", this means only DOM/browser interaction simulating a real user.

**Persistence / rendering from JSON:** node `position {x, y}`, edges, and
viewport are persisted into the PageContext `canvas` object (see
[schemas/page-context.md](../schemas/page-context.md)) on save. Opening a
PageContext rebuilds React Flow `nodes` by joining `steps[]` (node `data`) with
`canvas.nodes[]` (position by `stepId`) and `canvas.edges[]`. If layout is
missing, the canvas falls back to an auto vertical chain.

**View type is a transient UI mode only.** Switching the Recorder/Manual tabs
does not change the saved JSON; both tabs render the same graph. Nothing about
the active view is persisted.

This /automations page has a control bar rendered above the tabs containing a
Test button and a button that opens the Expandable panel dialog (see below).

##### Expandable panel

The Expandable panel is a **UI dialog** (shadcn `Dialog`), not an inline
sidebar:

- **Fixed width/height** with `overflow-y: auto` (scrolls vertically when
  content exceeds the height). No pagination.
- **Tabs:** Tab 1 "DOM Actions", Tab 2 "Commands".
- Content is a **grid of icon buttons, 6 per row**. Each button is a catalog
  action (icon + label) sourced directly from
  [schemas/action-catalog.md](../schemas/action-catalog.md):
  - "Commands" tab = OS Commands + Browser Commands (mouse moves A -> B, clicks,
    keyboard input, scroll, reload, go back, go forward, redirect/navigate).
  - "DOM Actions" tab = DOM Interactions / Data (select elements, query
    properties from HTMLElements or the `document` instance, extract, delays).
- Clicking an action adds it as a node to the React Flow canvas; actions can
  also be dragged onto the canvas.

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
`resolver.title`. See [features/captcha-resolvers.md](captcha-resolvers.md) for
types/interfaces.

#### LLM Integration, `/llm-integrations`

This is a page that allows the user to setup what LLM intergration the LLM
module will use. In this case, add 2 intergrations for OpenROuter and Local
intergration via ollama. The default intergration will be OpenROuter and use the
OpenROuter/auto model.

The UI of this page will be a card UI with each card showing the intergration,
in this case, the card should show the intergreation name and its status
"COnfigured / not configured".

Open clicking a card will uopen up a UI dialog to confirgure the intergration
which will then persist the credentials securliy locally and update/notify the
backend what llm intergration to use when processing events that need LLM
functionality

## User actions streaming

Component working with the browser extension to record click/keyboard history on
DOM elements (macro streaming/replaying). User clicks record for a pageContext;
app opens a browser window with the extension, navigates to the domain, and
records mouse/keyboard events. The desktop view shows entries like
`Button#someButton` with `clientBoundingRect` details.

## Acceptance

- [ ] All routes in this spec render (`/`, `/browser-config`, `/automations`,
      `/integrations`, `/tasks`, `/llm-integration`, `/captcha-resolvers`)
- [ ] Global control bar and sidebar navigation work
- [ ] Automation cards show domain, favicon, and description on homepage
- [ ] Cycle dropdown in control bar runs selected automation group
- [ ] Session overlay dims screen (0.6 opacity) without blocking input; Escape
      terminates UserClient
- [ ] Demo/guide shown on first app start
- [ ] Recorder and Manual tabs share one React Flow canvas; nodes connected by
      arrows in order
- [ ] Node positions/edges persist to PageContext `canvas` and re-render on open
- [ ] Switching Recorder/Manual does not alter saved JSON (transient view)
- [ ] Expandable panel dialog: fixed size, scroll-y auto, tabs, 6-per-row grid
      of catalog actions; clicking/dragging adds a node
