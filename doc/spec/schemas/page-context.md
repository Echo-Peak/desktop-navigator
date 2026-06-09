---
id: SPEC-SCH-001
title: Page Context Schema
status: draft
depends_on:
  - SPEC-ARCH-001
  - SPEC-ARCH-002
implements: []
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

## Open Questions

- Env vars / creds protection: OS keychain/credential provider? User enters
  password/email in the Tauri UI; values must not be stored in plaintext on
  disk.

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

```typescript
export interface PageContext {
  domain: string;
  description: string;
  schemaVersion: string;
  env: PageContextEnv;
  steps: AutomationStep[];
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
  action: ActionPayload; // Discriminated union of action types
  timeoutMs?: number; // Max time to wait for completion (Default: 5000)
  optional?: boolean; // If true, failure skips to next step
  retries?: number; // Number of times to retry before failing
}

export type ActionPayload =
  // --- Navigation & Flow ---
  | { type: "navigate"; url: string }
  | { type: "wait"; condition: WaitCondition,  skipAfter: number } // skipAfter: 10s (default)

  // --- OS-Level Inputs (Via Input Controller / enigo) ---
  | { type: "click"; selector: string; offset?: { x: number; y: number } }
  | { type: "type"; selector: string; value: string; isSecret?: boolean }
  | { type: "keyboardShortcut"; keys: string[] }
  {
      type: "moveMouse",
      x: number,
      y: number,
      includeRandomness: boolean // default false
    }

  // --- Data Extraction & Processing ---
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
      type: "aggregateStrings";
      inputVariable: string; // Targets array saved from extractCollection
      template: string; // String template with {{keys}}
      joinWith: string; // Delimiter to join array (e.g., "\n\n")
      saveToVariable: string;
    } ;


export type WaitCondition =
  | { type: "time"; ms: number }
  | { type: "elementVisible"; selector: string }
  | { type: "elementHidden"; selector: string };
```

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

- [ ] TypeScript types in this spec are reflected in Rust/TS schema types
- [ ] BrowserContext and PageContext JSON validate against `schemaVersion`
- [ ] All `ActionPayload` variants parse and dispatch correctly
- [ ] Example payload in this spec runs end-to-end via the execution engine
