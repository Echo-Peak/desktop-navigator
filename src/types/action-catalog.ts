export type AutomationAction =
  | { type: "navigate"; url: string }
  | { type: "reload" }
  | { type: "goBack" }
  | { type: "goForward" }
  | { type: "moveMouse"; x: number; y: number; includeRandomness?: boolean }
  | { type: "click"; selector: string; offset?: { x: number; y: number } }
  | { type: "doubleClick"; selector: string; offset?: { x: number; y: number } }
  | { type: "rightClick"; selector: string; offset?: { x: number; y: number } }
  | { type: "type"; selector: string; value: string; isSecret?: boolean }
  | { type: "keyboardShortcut"; keys: string[] }
  | { type: "scroll"; selector?: string; deltaX?: number; deltaY?: number }
  | { type: "waitFor"; condition: WaitCondition; skipAfter?: number }
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
      selector?: string;
      property: string;
      saveToVariable: string;
    }
  | {
      type: "aggregateStrings";
      inputVariable: string;
      template: string;
      joinWith: string;
      saveToVariable: string;
    };

export type WaitCondition =
  | { type: "time"; ms: number }
  | { type: "elementVisible"; selector: string }
  | { type: "elementHidden"; selector: string };

export type ActionType = AutomationAction["type"];

export type ActionCategory = "browser" | "os" | "dom-data";

export interface ActionMeta {
  type: ActionType;
  category: ActionCategory;
  label: string;
}

export const ACTION_CATALOG: ActionMeta[] = [
  { type: "navigate", category: "browser", label: "Navigate" },
  { type: "reload", category: "browser", label: "Reload" },
  { type: "goBack", category: "browser", label: "Go Back" },
  { type: "goForward", category: "browser", label: "Go Forward" },
  { type: "moveMouse", category: "os", label: "Move Mouse" },
  { type: "click", category: "os", label: "Click" },
  { type: "doubleClick", category: "os", label: "Double Click" },
  { type: "rightClick", category: "os", label: "Right Click" },
  { type: "type", category: "os", label: "Type" },
  { type: "keyboardShortcut", category: "os", label: "Keyboard Shortcut" },
  { type: "scroll", category: "os", label: "Scroll" },
  { type: "waitFor", category: "dom-data", label: "Wait For" },
  { type: "extract", category: "dom-data", label: "Extract" },
  { type: "extractCollection", category: "dom-data", label: "Extract Collection" },
  { type: "queryProperty", category: "dom-data", label: "Query Property" },
  { type: "aggregateStrings", category: "dom-data", label: "Aggregate Strings" },
];
