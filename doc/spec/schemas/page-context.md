---
id: SPEC-SCH-001
title: Page Context Schema
status: implemented
depends_on:
  - SPEC-ARCH-001
  - SPEC-ARCH-002
  - SPEC-SCH-002
implements:
  - crates/core/src/schema.rs
  - src/types/page-context.ts
  - src/lib/storage.ts
  - src/lib/browserContext.ts
  - src/components/canvas/FlowCanvas.tsx
---

# Goal

Create a JSON schema system that allows the user to modify the behavior of the
app. The system includes two contexts: browser context and page context.

There is one schema for the browser context. There are many schemas for page
contexts.

Schema files must be versioned for major app changes.

**Browser context** — outermost control:

- Browser path to use/spawn
- Browser flags
- Browser dimensions
- Integrations (Slack webhook, email)

**Page context** — page/domain level:

- `steps`: array of actions (see ## Steps)
- Env vars/creds for login

# Requirements

## Storage

Schema files are stored on the filesystem within the user directory. See
[architecture/data-management.md](../architecture/data-management.md).

## Credential protection

Env vars / creds are protected by the Secret Vault (OS keychain) defined in
[architecture/execution-engine.md](../architecture/execution-engine.md) section
2.5. The user enters password/email in the Tauri UI; values are stored in the OS
keychain (macOS Keychain, Windows Credential Manager, Linux Secret Service) and
referenced from the schema as `env:KEY` — never written to disk in plaintext.

## Steps

A sequence of actions the UserClient (keyboard/mouse) and backend process on the
page.

### Types and Interfaces

#### BrowserContext

Defines how the browser is spawned and global execution rules.

```typescript
export interface BrowserContext {
  schemaVersion: string;
  browserConfig: {
    incognito: boolean; // default is false
    hadAudio: boolean; // default is true
    isFullscreen: boolean; // default is true
    executablePath?: string; // Optional: auto-detect if omitted
    width: number; // Enforced width (e.g., 1280)
    height: number; // Enforced height (e.g., 720)
    xPosition: number; // Default: 0
    yPosition: number; // Default: 0
    flags?: string[]; // Additional Chromium flags
  };
  integrations?: {
    default: string;
    config: IntegrationConfig;
  };
}

export interface Integration {
  type: "webhook" | "file";
  destination: string; // URL or Filepath. Supports env/secret variables
  payloadTemplate: Record<string, any>; // JSON payload template
}
```

#### PageContext

Defines a page (specific URL).

`steps[]` is the linear execution source of truth (ordered). `canvas` is a
presentation-only layout used to re-render the node-graph diagram; it never
affects execution order and is ignored by the execution engine.

```typescript
export interface PageContext {
  domain: string;
  description: string;
  schemaVersion: string;
  env: PageContextEnv;
  steps: AutomationStep[]; // execution source of truth (ordered)
  canvas?: AutomationCanvas; // presentation-only layout for the diagram
  output?: IntegrationConfig;
}

export interface PageContextEnv {
  PASSWORD?: string;
  EMAIL?: string;
  [envVar: string]: string;
}

export interface AutomationStep {
  id: string; // Unique identifier for logs
  description?: string; // Human readable context
  action: AutomationAction; // see schemas/action-catalog.md (single source of truth)
  timeoutMs?: number; // Max time to wait for completion (Default: 5000)
  optional?: boolean; // If true, failure skips to next step
  retries?: number; // Number of times to retry before failing
}
```

The `action` field is an `AutomationAction` from the canonical
[schemas/action-catalog.md](action-catalog.md). All action and `WaitCondition`
types live there; this spec does not redefine them.

#### AutomationCanvas

Presentation-only layout for the node-graph diagram (React Flow / `@xyflow/react`).
Each node maps 1:1 to a step via `stepId`.

```typescript
export interface AutomationCanvas {
  // one node per step, keyed by AutomationStep.id
  nodes: { stepId: string; position: { x: number; y: number } }[];
  // visual connectors; for v1 these mirror linear step order (A -> B -> C)
  edges: { id: string; source: string; target: string }[];
  viewport?: { x: number; y: number; zoom: number };
}
```

- Each `canvas.nodes[].stepId` MUST reference an existing `steps[].id`. The UI
  prunes stale node entries whose step was deleted.
- If `canvas` is absent, or a step has no node entry, the UI auto-lays-out a
  default vertical chain.
- Maps directly onto `@xyflow/react`: a React Flow node is
  `{ id: stepId, position, data: <the AutomationStep> }`; `canvas.edges` map to
  React Flow edges.
- The execution engine ignores `canvas` entirely (see
  [architecture/execution-engine.md](../architecture/execution-engine.md)).

### Example payload

```json
{
  "domain": "job-board.example",
  "schemaVersion": "1.0",
  "browserConfig": {
    "width": 1280,
    "height": 720,
    "xPosition": 0,
    "yPosition": 0
  },
  "steps": [
    {
      "id": "login_type",
      "action": {
        "type": "type",
        "selector": "#password",
        "value": "env:APP_SECRET_PASSWORD",
        "isSecret": true
      }
    },
    {
      "id": "scrape_items",
      "action": {
        "type": "extractCollection",
        "containerSelector": ".job-list",
        "itemSelector": ".job-card",
        "extract": {
          "title": { "selector": "h2", "extractType": "text" },
          "link": {
            "selector": "a",
            "extractType": "attribute",
            "attributeName": "href"
          }
        },
        "saveToVariable": "scraped_jobs"
      }
    },
    {
      "id": "format_for_slack",
      "action": {
        "type": "aggregateStrings",
        "inputVariable": "scraped_jobs",
        "template": "• *{{title}}*\n  {{link}}",
        "joinWith": "\n\n",
        "saveToVariable": "slack_message_body"
      }
    }
  ],
  "canvas": {
    "nodes": [
      { "stepId": "login_type", "position": { "x": 0, "y": 0 } },
      { "stepId": "scrape_items", "position": { "x": 0, "y": 160 } },
      { "stepId": "format_for_slack", "position": { "x": 0, "y": 320 } }
    ],
    "edges": [
      { "id": "e1", "source": "login_type", "target": "scrape_items" },
      { "id": "e2", "source": "scrape_items", "target": "format_for_slack" }
    ],
    "viewport": { "x": 0, "y": 0, "zoom": 1 }
  },
  "integrations": [
    {
      "type": "webhook",
      "destination": "env:SLACK_WEBHOOK_URL",
      "payloadTemplate": {
        "text": "New Jobs:\n\n{{slack_message_body}}"
      }
    }
  ]
}
```

## Acceptance

- [x] TypeScript types in this spec are reflected in Rust/TS schema types
- [x] BrowserContext and PageContext JSON validate against `schemaVersion`
- [x] `AutomationStep.action` uses the `AutomationAction` union from
      [action-catalog.md](action-catalog.md); no action types are redefined here
- [x] `canvas` round-trips: positions/edges persist and re-render the diagram
- [x] `canvas.nodes[].stepId` referential integrity enforced (stale nodes pruned)
- [x] Example payload in this spec runs end-to-end via the execution engine
