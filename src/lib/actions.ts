import {
  ArrowLeft,
  ArrowRight,
  Combine,
  Compass,
  Download,
  Keyboard,
  ListTree,
  type LucideIcon,
  MousePointer,
  MousePointer2,
  MousePointerClick,
  MoveVertical,
  RotateCw,
  Search,
  Timer
} from "lucide-react";
import type { ActionType, AutomationAction } from "@/types/action-catalog";

export const ACTION_ICONS: Record<ActionType, LucideIcon> = {
  navigate: Compass,
  reload: RotateCw,
  goBack: ArrowLeft,
  goForward: ArrowRight,
  moveMouse: MousePointer2,
  click: MousePointerClick,
  doubleClick: MousePointerClick,
  rightClick: MousePointer,
  type: Keyboard,
  keyboardShortcut: Keyboard,
  scroll: MoveVertical,
  waitFor: Timer,
  extract: Download,
  extractCollection: ListTree,
  queryProperty: Search,
  aggregateStrings: Combine
};

export function defaultAction(type: ActionType): AutomationAction {
  switch (type) {
    case "navigate":
      return { type, url: "https://" };
    case "reload":
    case "goBack":
    case "goForward":
      return { type };
    case "moveMouse":
      return { type, x: 0, y: 0, includeRandomness: true };
    case "click":
    case "doubleClick":
    case "rightClick":
      return { type, selector: "" };
    case "type":
      return { type, selector: "", value: "" };
    case "keyboardShortcut":
      return { type, keys: [] };
    case "scroll":
      return { type, deltaY: 0 };
    case "waitFor":
      return { type, condition: { type: "time", ms: 1000 } };
    case "extract":
      return { type, selector: "", extractType: "text", saveToVariable: "" };
    case "extractCollection":
      return {
        type,
        containerSelector: "",
        itemSelector: "",
        extract: {},
        saveToVariable: ""
      };
    case "queryProperty":
      return { type, target: "element", property: "", saveToVariable: "" };
    case "aggregateStrings":
      return {
        type,
        inputVariable: "",
        template: "",
        joinWith: "",
        saveToVariable: ""
      };
  }
}

export function newStepId(): string {
  return `step_${Date.now().toString(36)}_${Math.random()
    .toString(36)
    .slice(2, 7)}`;
}
