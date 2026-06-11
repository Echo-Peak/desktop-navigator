use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::schema::{ExtractField, Offset, QueryTarget};

use super::error::EngineResult;

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct DomRect {
    pub x: f64,
    pub y: f64,
    pub width: f64,
    pub height: f64,
    pub top: f64,
    pub left: f64,
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct WindowGeometry {
    pub screen_x: f64,
    pub screen_y: f64,
    pub chrome_height: f64,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ScreenPoint {
    pub x: i32,
    pub y: i32,
}

pub fn rect_to_screen(rect: &DomRect, window: &WindowGeometry, offset: Option<&Offset>) -> ScreenPoint {
    let (ox, oy) = offset.map(|o| (o.x, o.y)).unwrap_or((0.0, 0.0));
    let x = window.screen_x + rect.left + rect.width / 2.0 + ox;
    let y = window.screen_y + window.chrome_height + rect.top + rect.height / 2.0 + oy;
    ScreenPoint {
        x: x.round() as i32,
        y: y.round() as i32,
    }
}

pub trait DomBridge {
    fn element_geometry(&mut self, selector: &str) -> EngineResult<(DomRect, WindowGeometry)>;
    fn get_text(&mut self, selector: &str) -> EngineResult<String>;
    fn get_attribute(&mut self, selector: &str, attribute: &str) -> EngineResult<String>;
    fn query_property(
        &mut self,
        target: QueryTarget,
        selector: Option<&str>,
        property: &str,
    ) -> EngineResult<Value>;
    fn is_visible(&mut self, selector: &str) -> EngineResult<bool>;
    fn extract_collection(
        &mut self,
        container_selector: &str,
        item_selector: &str,
        fields: &std::collections::BTreeMap<String, ExtractField>,
    ) -> EngineResult<Value>;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn maps_rect_center_to_absolute_screen_coords() {
        let rect = DomRect {
            x: 100.0,
            y: 200.0,
            width: 40.0,
            height: 20.0,
            top: 200.0,
            left: 100.0,
        };
        let window = WindowGeometry {
            screen_x: 0.0,
            screen_y: 0.0,
            chrome_height: 80.0,
        };
        let point = rect_to_screen(&rect, &window, None);
        assert_eq!(point.x, 120);
        assert_eq!(point.y, 290);
    }

    #[test]
    fn applies_window_offset_and_action_offset() {
        let rect = DomRect {
            x: 0.0,
            y: 0.0,
            width: 10.0,
            height: 10.0,
            top: 0.0,
            left: 0.0,
        };
        let window = WindowGeometry {
            screen_x: 50.0,
            screen_y: 30.0,
            chrome_height: 70.0,
        };
        let offset = Offset { x: 3.0, y: -4.0 };
        let point = rect_to_screen(&rect, &window, Some(&offset));
        assert_eq!(point.x, 58);
        assert_eq!(point.y, 101);
    }
}
