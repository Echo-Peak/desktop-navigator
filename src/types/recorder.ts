export type RecordedEventType =
  | "click"
  | "dblclick"
  | "type"
  | "keypress"
  | "navigate"
  | "focus";

export interface ElementDescriptor {
  tagName: string;
  id?: string;
  classList: string[];
  name?: string;
  type?: string;
  placeholder?: string;
  ariaLabel?: string;
  text?: string;
  href?: string;
  selector: string;
  xpath: string;
  boundingRect: {
    x: number;
    y: number;
    width: number;
    height: number;
    top: number;
    right: number;
    bottom: number;
    left: number;
  };
}

export interface RecordedEvent {
  id: string;
  sequence: number;
  sessionId: string;
  timestamp: number;
  type: RecordedEventType;
  pageUrl: string;
  element?: ElementDescriptor;
  value?: string;
  sensitiveInput?: boolean;
  key?: string;
}

export type RecorderClientMessage =
  | { msg: "event"; payload: RecordedEvent }
  | { msg: "ready"; sessionId: string }
  | { msg: "browser_closed" };

export type RecorderServerMessage =
  | { msg: "start"; sessionId: string }
  | { msg: "pause" }
  | { msg: "stop" };
