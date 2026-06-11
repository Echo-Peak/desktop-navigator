use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum ExtractType {
    Text,
    Attribute,
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum QueryTarget {
    Element,
    Document,
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct Offset {
    pub x: f64,
    pub y: f64,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ExtractField {
    pub selector: String,
    pub extract_type: ExtractType,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub attribute_name: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "camelCase", rename_all_fields = "camelCase")]
pub enum WaitCondition {
    Time { ms: u64 },
    ElementVisible { selector: String },
    ElementHidden { selector: String },
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "camelCase", rename_all_fields = "camelCase")]
pub enum AutomationAction {
    Navigate {
        url: String,
    },
    Reload,
    GoBack,
    GoForward,
    MoveMouse {
        x: f64,
        y: f64,
        #[serde(default)]
        include_randomness: bool,
    },
    Click {
        selector: String,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        offset: Option<Offset>,
    },
    DoubleClick {
        selector: String,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        offset: Option<Offset>,
    },
    RightClick {
        selector: String,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        offset: Option<Offset>,
    },
    Type {
        selector: String,
        value: String,
        #[serde(default)]
        is_secret: bool,
    },
    KeyboardShortcut {
        keys: Vec<String>,
    },
    Scroll {
        #[serde(default, skip_serializing_if = "Option::is_none")]
        selector: Option<String>,
        #[serde(default)]
        delta_x: f64,
        #[serde(default)]
        delta_y: f64,
    },
    WaitFor {
        condition: WaitCondition,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        skip_after: Option<u64>,
    },
    Extract {
        selector: String,
        extract_type: ExtractType,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        attribute_name: Option<String>,
        save_to_variable: String,
    },
    ExtractCollection {
        container_selector: String,
        item_selector: String,
        extract: BTreeMap<String, ExtractField>,
        save_to_variable: String,
    },
    QueryProperty {
        target: QueryTarget,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        selector: Option<String>,
        property: String,
        save_to_variable: String,
    },
    AggregateStrings {
        input_variable: String,
        template: String,
        join_with: String,
        save_to_variable: String,
    },
}

impl AutomationAction {
    pub fn type_name(&self) -> &'static str {
        match self {
            AutomationAction::Navigate { .. } => "navigate",
            AutomationAction::Reload => "reload",
            AutomationAction::GoBack => "goBack",
            AutomationAction::GoForward => "goForward",
            AutomationAction::MoveMouse { .. } => "moveMouse",
            AutomationAction::Click { .. } => "click",
            AutomationAction::DoubleClick { .. } => "doubleClick",
            AutomationAction::RightClick { .. } => "rightClick",
            AutomationAction::Type { .. } => "type",
            AutomationAction::KeyboardShortcut { .. } => "keyboardShortcut",
            AutomationAction::Scroll { .. } => "scroll",
            AutomationAction::WaitFor { .. } => "waitFor",
            AutomationAction::Extract { .. } => "extract",
            AutomationAction::ExtractCollection { .. } => "extractCollection",
            AutomationAction::QueryProperty { .. } => "queryProperty",
            AutomationAction::AggregateStrings { .. } => "aggregateStrings",
        }
    }

    pub fn category(&self) -> ActionCategory {
        match self {
            AutomationAction::Navigate { .. }
            | AutomationAction::Reload
            | AutomationAction::GoBack
            | AutomationAction::GoForward => ActionCategory::Browser,
            AutomationAction::MoveMouse { .. }
            | AutomationAction::Click { .. }
            | AutomationAction::DoubleClick { .. }
            | AutomationAction::RightClick { .. }
            | AutomationAction::Type { .. }
            | AutomationAction::KeyboardShortcut { .. }
            | AutomationAction::Scroll { .. } => ActionCategory::Os,
            AutomationAction::WaitFor { .. }
            | AutomationAction::Extract { .. }
            | AutomationAction::ExtractCollection { .. }
            | AutomationAction::QueryProperty { .. }
            | AutomationAction::AggregateStrings { .. } => ActionCategory::DomData,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ActionCategory {
    Browser,
    Os,
    DomData,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AutomationStep {
    pub id: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    pub action: AutomationAction,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub timeout_ms: Option<u64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub optional: Option<bool>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub retries: Option<u32>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CanvasNode {
    #[serde(rename = "stepId")]
    pub step_id: String,
    pub position: CanvasPoint,
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct CanvasPoint {
    pub x: f64,
    pub y: f64,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CanvasEdge {
    pub id: String,
    pub source: String,
    pub target: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AutomationCanvas {
    pub nodes: Vec<CanvasNode>,
    pub edges: Vec<CanvasEdge>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub viewport: Option<CanvasViewport>,
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct CanvasViewport {
    pub x: f64,
    pub y: f64,
    pub zoom: f64,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Integration {
    #[serde(rename = "type")]
    pub kind: IntegrationKind,
    pub destination: String,
    #[serde(default)]
    pub payload_template: serde_json::Value,
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum IntegrationKind {
    Webhook,
    File,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PageContext {
    pub domain: String,
    #[serde(default)]
    pub description: String,
    pub schema_version: String,
    #[serde(default)]
    pub env: BTreeMap<String, String>,
    pub steps: Vec<AutomationStep>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub canvas: Option<AutomationCanvas>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub output: Option<Vec<Integration>>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BrowserConfig {
    #[serde(default)]
    pub incognito: bool,
    #[serde(default = "default_true")]
    pub had_audio: bool,
    #[serde(default = "default_true")]
    pub is_fullscreen: bool,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub executable_path: Option<String>,
    pub width: u32,
    pub height: u32,
    #[serde(default)]
    pub x_position: i32,
    #[serde(default)]
    pub y_position: i32,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub flags: Option<Vec<String>>,
}

fn default_true() -> bool {
    true
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BrowserContext {
    pub schema_version: String,
    pub browser_config: BrowserConfig,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn action_roundtrips_with_catalog_tags() {
        let json = r##"{"type":"click","selector":"#go"}"##;
        let action: AutomationAction = serde_json::from_str(json).unwrap();
        assert_eq!(action.type_name(), "click");
        let back = serde_json::to_string(&action).unwrap();
        assert!(back.contains("\"type\":\"click\""));
    }

    #[test]
    fn type_action_uses_camel_case_fields() {
        let json = r##"{"type":"type","selector":"#pw","value":"env:PW","isSecret":true}"##;
        let action: AutomationAction = serde_json::from_str(json).unwrap();
        match action {
            AutomationAction::Type {
                is_secret, value, ..
            } => {
                assert!(is_secret);
                assert_eq!(value, "env:PW");
            }
            _ => panic!("wrong variant"),
        }
    }

    #[test]
    fn unit_browser_actions_parse() {
        for (json, name) in [
            (r#"{"type":"reload"}"#, "reload"),
            (r#"{"type":"goBack"}"#, "goBack"),
            (r#"{"type":"goForward"}"#, "goForward"),
        ] {
            let a: AutomationAction = serde_json::from_str(json).unwrap();
            assert_eq!(a.type_name(), name);
        }
    }

    #[test]
    fn page_context_example_parses_and_ignores_canvas_for_steps() {
        let json = r##"{
            "domain": "job-board.example",
            "schemaVersion": "1.0",
            "steps": [
                {"id":"login_type","action":{"type":"type","selector":"#password","value":"env:APP_SECRET_PASSWORD","isSecret":true}},
                {"id":"scrape","action":{"type":"extractCollection","containerSelector":".job-list","itemSelector":".job-card","extract":{"title":{"selector":"h2","extractType":"text"}},"saveToVariable":"jobs"}}
            ],
            "canvas": {
                "nodes": [{"stepId":"login_type","position":{"x":0,"y":0}}],
                "edges": [{"id":"e1","source":"login_type","target":"scrape"}]
            }
        }"##;
        let page: PageContext = serde_json::from_str(json).unwrap();
        assert_eq!(page.steps.len(), 2);
        assert_eq!(page.steps[0].id, "login_type");
        assert!(page.canvas.is_some());
    }

    #[test]
    fn extract_type_serializes_lowercase() {
        let f = ExtractField {
            selector: "h2".into(),
            extract_type: ExtractType::Text,
            attribute_name: None,
        };
        let s = serde_json::to_string(&f).unwrap();
        assert!(s.contains("\"extractType\":\"text\""));
    }
}
