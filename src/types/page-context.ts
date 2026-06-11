import type { AutomationAction } from "./action-catalog";

export interface Integration {
  type: "webhook" | "file";
  destination: string;
  payloadTemplate: Record<string, unknown>;
}

export type IntegrationConfig = Integration[];

export interface BrowserContext {
  schemaVersion: string;
  browserConfig: {
    incognito: boolean;
    hadAudio: boolean;
    isFullscreen: boolean;
    executablePath?: string;
    width: number;
    height: number;
    xPosition: number;
    yPosition: number;
    flags?: string[];
  };
  integrations?: {
    default: string;
    config: IntegrationConfig;
  };
}

export interface PageContextEnv {
  PASSWORD?: string;
  EMAIL?: string;
  [envVar: string]: string | undefined;
}

export interface AutomationStep {
  id: string;
  description?: string;
  action: AutomationAction;
  timeoutMs?: number;
  optional?: boolean;
  retries?: number;
}

export interface AutomationCanvas {
  nodes: { stepId: string; position: { x: number; y: number } }[];
  edges: { id: string; source: string; target: string }[];
  viewport?: { x: number; y: number; zoom: number };
}

export interface PageContext {
  domain: string;
  description: string;
  schemaVersion: string;
  env: PageContextEnv;
  steps: AutomationStep[];
  canvas?: AutomationCanvas;
  output?: IntegrationConfig;
}
