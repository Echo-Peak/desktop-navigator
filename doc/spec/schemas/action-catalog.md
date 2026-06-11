---
id: SPEC-SCH-002
title: Action Catalog
status: approved
depends_on:
  - SPEC-PROD-001
implements: []
---

# Goal

Define the single canonical catalog of every action/command that can be used to
build a page automation. This catalog is the one source of truth consumed by:

- the page schema ([schemas/page-context.md](page-context.md)) — `AutomationStep.action`
- the desktop UI ([features/desktop-ui.md](../features/desktop-ui.md)) — the
  Manual editor panel and the node-graph canvas
- the macro recorder ([features/macro-recorder.md](../features/macro-recorder.md))
  — recorded events map onto these actions
- the execution engine
  ([architecture/execution-engine.md](../architecture/execution-engine.md)) —
  dispatches each action to its executor subsystem

Any new action is added here first; every other spec references this list rather
than defining its own.

# Requirements

## Categories

Actions are grouped into four categories. Each action declares the subsystem that
executes it.

### Browser Commands (Browser Manager)

- `navigate` — load a URL (redirect)
- `reload` — reload the current page
- `goBack` — navigate back in history
- `goForward` — navigate forward in history

### OS Commands (Input Controller / enigo)

- `moveMouse` — humanized A->B pointer move (never teleports)
- `click` — left click
- `doubleClick` — double left click
- `rightClick` — right click (context menu)
- `type` — type a string into a focused/selected field
- `keyboardShortcut` — press one key or a key combination
- `scroll` — scroll the page or an element

### DOM Interactions / Data (DOM-to-OS Bridge + Schema Engine)

- `waitFor` — wait for a `WaitCondition`
- `extract` — read text/attribute from one element into a variable
- `extractCollection` — read repeated items into an array variable
- `queryProperty` — read a property from an `HTMLElement` or the `document`
  instance into a variable
- `aggregateStrings` — template + join an array variable into a string

## Unified `AutomationAction`

This discriminated union is the authoritative action shape. It is referenced by
`AutomationStep.action` in [page-context.md](page-context.md).

```typescript
export type AutomationAction =
  // --- Browser Commands (Browser Manager) ---
  | { type: "navigate"; url: string }
  | { type: "reload" }
  | { type: "goBack" }
  | { type: "goForward" }

  // --- OS Commands (Input Controller / enigo) ---
  | { type: "moveMouse"; x: number; y: number; includeRandomness?: boolean } // default false
  | { type: "click"; selector: string; offset?: { x: number; y: number } }
  | { type: "doubleClick"; selector: string; offset?: { x: number; y: number } }
  | { type: "rightClick"; selector: string; offset?: { x: number; y: number } }
  | { type: "type"; selector: string; value: string; isSecret?: boolean }
  | { type: "keyboardShortcut"; keys: string[] } // single key = one-element array
  | { type: "scroll"; selector?: string; deltaX?: number; deltaY?: number }

  // --- DOM Interactions / Data ---
  | { type: "waitFor"; condition: WaitCondition; skipAfter?: number } // skipAfter ms, default 10000
  | {
      type: "extract";
      selector: string;
      extractType: "text" | "attribute";
      attributeName?: string;
      saveToVariable: string;
    }
  | {
      type: "extractCollection";
      containerSelector: string;
      itemSelector: string;
      extract: Record<
        string,
        {
          selector: string;
          extractType: "text" | "attribute";
          attributeName?: string;
        }
      >;
      saveToVariable: string;
    }
  | {
      type: "queryProperty";
      target: "element" | "document";
      selector?: string; // required when target === "element"
      property: string; // e.g. "scrollHeight", "value", "title"
      saveToVariable: string;
    }
  | {
      type: "aggregateStrings";
      inputVariable: string; // array saved from extractCollection
      template: string; // string template with {{keys}}
      joinWith: string; // delimiter (e.g. "\n\n")
      saveToVariable: string;
    };

export type WaitCondition =
  | { type: "time"; ms: number }
  | { type: "elementVisible"; selector: string }
  | { type: "elementHidden"; selector: string };
```

## Reconciliation with prior vocabularies

The recorder and an earlier draft of the schema used names that this catalog
unifies:

- `press` (recorder) -> `keyboardShortcut` with a one-element `keys` array.
- `waitForElement` (recorder export) -> `waitFor` with
  `{ condition: { type: "elementVisible" } }`.
- `doubleClick`, `rightClick`, `scroll`, `queryProperty` are first-class actions
  here (previously missing from the schema union).

## Action metadata

The UI panel, node-graph canvas, and recorder mapping all derive from this table.
`Executor` is the subsystem in
[architecture/execution-engine.md](../architecture/execution-engine.md).

| Action | Category | Executor | Key params | Recorder-capturable |
| --- | --- | --- | --- | --- |
| `navigate` | Browser | Browser Manager | `url` | yes |
| `reload` | Browser | Browser Manager | — | no |
| `goBack` | Browser | Browser Manager | — | no |
| `goForward` | Browser | Browser Manager | — | no |
| `moveMouse` | OS | Input Controller | `x`, `y`, `includeRandomness?` | no |
| `click` | OS | Input Controller | `selector`, `offset?` | yes |
| `doubleClick` | OS | Input Controller | `selector`, `offset?` | yes |
| `rightClick` | OS | Input Controller | `selector`, `offset?` | yes |
| `type` | OS | Input Controller | `selector`, `value`, `isSecret?` | yes |
| `keyboardShortcut` | OS | Input Controller | `keys[]` | yes |
| `scroll` | OS | Input Controller | `selector?`, `deltaX?`, `deltaY?` | yes |
| `waitFor` | DOM/Data | DOM-to-OS Bridge | `condition`, `skipAfter?` | no |
| `extract` | DOM/Data | Schema Engine | `selector`, `extractType`, `saveToVariable` | no |
| `extractCollection` | DOM/Data | Schema Engine | `containerSelector`, `itemSelector`, `extract`, `saveToVariable` | no |
| `queryProperty` | DOM/Data | DOM-to-OS Bridge | `target`, `selector?`, `property`, `saveToVariable` | no |
| `aggregateStrings` | DOM/Data | Schema Engine | `inputVariable`, `template`, `joinWith`, `saveToVariable` | no |

## Acceptance

- [ ] `AutomationAction` is defined only in this spec; no other spec redefines
      the action union
- [ ] Every action lists its category and executor subsystem in the metadata
      table
- [ ] Recorder vocabulary (`press`, `waitForElement`, `dblclick`) maps onto
      catalog actions with no orphan names
- [ ] page-context, desktop-ui, macro-recorder, and execution-engine reference
      this catalog rather than defining their own list
