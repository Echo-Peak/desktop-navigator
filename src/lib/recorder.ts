import type { RecordedEvent } from "@/types/recorder";
import type { AutomationAction } from "@/types/action-catalog";
import type { AutomationCanvas, AutomationStep } from "@/types/page-context";
import { newStepId } from "./actions";

export function eventToAction(e: RecordedEvent): AutomationAction | null {
  switch (e.type) {
    case "click":
      return e.element ? { type: "click", selector: e.element.selector } : null;
    case "dblclick":
      return e.element
        ? { type: "doubleClick", selector: e.element.selector }
        : null;
    case "type":
      return e.element
        ? {
            type: "type",
            selector: e.element.selector,
            value: e.value ?? "",
            ...(e.sensitiveInput ? { isSecret: true } : {})
          }
        : null;
    case "keypress":
      return { type: "keyboardShortcut", keys: e.key ? [e.key] : [] };
    case "navigate":
      return { type: "navigate", url: e.pageUrl };
    case "focus":
      return null;
  }
}

export function eventDescription(e: RecordedEvent): string | undefined {
  const label = e.element?.text || e.element?.ariaLabel;
  if (e.type === "click" && label) return `Click '${label}'`;
  if (e.type === "type" && e.element)
    return `Type into ${e.element.name || e.element.selector}`;
  if (e.type === "navigate") return `Navigate to ${e.pageUrl}`;
  return undefined;
}

export function eventToStep(e: RecordedEvent): AutomationStep | null {
  const action = eventToAction(e);
  if (!action) return null;
  const description = eventDescription(e);
  return {
    id: newStepId(),
    action,
    ...(description ? { description } : {}),
    timeoutMs: 10000
  };
}

const NEEDS_WAIT = new Set(["click", "doubleClick", "type"]);

export function eventsToSteps(events: RecordedEvent[]): AutomationStep[] {
  const steps: AutomationStep[] = [];
  for (const e of events) {
    const action = eventToAction(e);
    if (!action) continue;
    if (NEEDS_WAIT.has(action.type) && e.element) {
      steps.push({
        id: newStepId(),
        action: {
          type: "waitFor",
          condition: { type: "elementVisible", selector: e.element.selector }
        },
        timeoutMs: 10000
      });
    }
    const description = eventDescription(e);
    steps.push({
      id: newStepId(),
      action,
      ...(description ? { description } : {}),
      timeoutMs: 10000
    });
  }
  return steps;
}

export function stepsToCanvas(steps: AutomationStep[]): AutomationCanvas {
  return {
    nodes: steps.map((s, i) => ({
      stepId: s.id,
      position: { x: 140, y: i * 120 + 40 }
    })),
    edges: steps.slice(1).map((s, i) => ({
      id: `e_${steps[i].id}_${s.id}`,
      source: steps[i].id,
      target: s.id
    }))
  };
}

export function hasSensitive(events: RecordedEvent[]): boolean {
  return events.some((e) => e.sensitiveInput);
}
