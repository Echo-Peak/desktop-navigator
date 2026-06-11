export interface CaptchaResolver {
  id: string;
  title: string;
  schemaVersion: string;
  description?: string;
  detection: DetectionConfig;
  solving: SolvingConfig;
}

export interface DetectionConfig {
  matchMode: "all" | "any";
  conditions: DetectionStep[];
}

export type DetectionStep =
  | { type: "pageTitle"; match: StringMatch }
  | { type: "elementExists"; selector: string }
  | { type: "elementText"; selector: string; match: StringMatch }
  | { type: "urlMatches"; match: StringMatch };

export type StringMatch =
  | { mode: "equals"; value: string }
  | { mode: "contains"; value: string }
  | { mode: "regex"; pattern: string; flags?: string };

export interface SolvingConfig {
  steps: SolvingStep[];
}

export type SolvingStep =
  | {
      type: "queryElement";
      selector: string;
      useParentNode?: boolean;
      saveToVariable: string;
    }
  | { type: "getBoundingRect"; targetVariable: string; saveToVariable: string }
  | { type: "getPageTitle"; saveToVariable: string }
  | { type: "moveMouse"; x: ValueExpr; y: ValueExpr; includeRandomness?: boolean }
  | { type: "click"; x: ValueExpr; y: ValueExpr }
  | {
      type: "captureScreenshot";
      region: "viewport" | "element";
      targetVariable?: string;
      saveToVariable: string;
    }
  | {
      type: "visionLocate";
      imageVariable: string;
      prompt: string;
      saveToVariable: string;
    }
  | { type: "wait"; ms: number }
  | { type: "setVariable"; name: string; value: ValueExpr }
  | {
      type: "checkCondition";
      condition: Condition;
      onFail: "break" | "fail" | "continue";
    }
  | { type: "loop"; while: Condition; maxIterations: number; steps: SolvingStep[] }
  | { type: "break"; if?: Condition };

export type Condition =
  | { type: "pageTitle"; match: StringMatch }
  | { type: "elementExists"; selector: string }
  | { type: "elementHidden"; selector: string }
  | {
      type: "compare";
      left: ValueExpr;
      op: "<" | "<=" | ">" | ">=" | "==" | "!=";
      right: ValueExpr;
    };

export type ValueExpr =
  | number
  | string
  | boolean
  | { ref: string }
  | { op: "add" | "sub" | "mul" | "div"; args: ValueExpr[] };
