pub mod bridge;
pub mod browser;
pub mod dispatcher;
pub mod error;
pub mod input;
pub mod secret;
pub mod state;

pub use bridge::{rect_to_screen, DomBridge, DomRect, ScreenPoint, WindowGeometry};
pub use browser::{launch_args, BrowserManager};
pub use dispatcher::{integration_payload, Clock, Engine, SystemClock};
pub use error::{EngineError, EngineResult};
pub use input::{plan_motion, InputSink, Motion, Rng, UserClient};
pub use secret::{resolve_value, InMemorySecrets, SecretResolver};
pub use state::StateMap;

#[cfg(test)]
mod e2e_tests {
    use super::*;
    use crate::schema::PageContext;
    use std::collections::BTreeMap;

    #[derive(Default)]
    struct MockBrowser {
        log: Vec<String>,
    }
    impl BrowserManager for MockBrowser {
        fn navigate(&mut self, url: &str) -> EngineResult<()> {
            self.log.push(format!("navigate:{url}"));
            Ok(())
        }
        fn reload(&mut self) -> EngineResult<()> {
            self.log.push("reload".into());
            Ok(())
        }
        fn go_back(&mut self) -> EngineResult<()> {
            Ok(())
        }
        fn go_forward(&mut self) -> EngineResult<()> {
            Ok(())
        }
    }

    #[derive(Default)]
    struct MockBridge;
    impl DomBridge for MockBridge {
        fn element_geometry(&mut self, _selector: &str) -> EngineResult<(DomRect, WindowGeometry)> {
            Ok((
                DomRect {
                    x: 10.0,
                    y: 20.0,
                    width: 100.0,
                    height: 40.0,
                    top: 20.0,
                    left: 10.0,
                },
                WindowGeometry {
                    screen_x: 0.0,
                    screen_y: 0.0,
                    chrome_height: 80.0,
                },
            ))
        }
        fn get_text(&mut self, _selector: &str) -> EngineResult<String> {
            Ok("Hello".into())
        }
        fn get_attribute(&mut self, _selector: &str, _attribute: &str) -> EngineResult<String> {
            Ok("http://link".into())
        }
        fn query_property(
            &mut self,
            _target: crate::schema::QueryTarget,
            _selector: Option<&str>,
            _property: &str,
        ) -> EngineResult<serde_json::Value> {
            Ok(serde_json::json!(1234))
        }
        fn is_visible(&mut self, _selector: &str) -> EngineResult<bool> {
            Ok(true)
        }
        fn extract_collection(
            &mut self,
            _container_selector: &str,
            _item_selector: &str,
            _fields: &BTreeMap<String, crate::schema::ExtractField>,
        ) -> EngineResult<serde_json::Value> {
            Ok(serde_json::json!([
                {"title": "Job A", "link": "http://a"},
                {"title": "Job B", "link": "http://b"}
            ]))
        }
    }

    #[derive(Default)]
    struct CountingSink {
        clicks: usize,
        typed: String,
    }
    impl InputSink for CountingSink {
        fn move_to(&mut self, _x: i32, _y: i32) -> EngineResult<()> {
            Ok(())
        }
        fn left_click(&mut self) -> EngineResult<()> {
            self.clicks += 1;
            Ok(())
        }
        fn double_click(&mut self) -> EngineResult<()> {
            Ok(())
        }
        fn right_click(&mut self) -> EngineResult<()> {
            Ok(())
        }
        fn scroll(&mut self, _dx: i32, _dy: i32) -> EngineResult<()> {
            Ok(())
        }
        fn key_combo(&mut self, _keys: &[String]) -> EngineResult<()> {
            Ok(())
        }
        fn type_text(&mut self, text: &str) -> EngineResult<()> {
            self.typed.push_str(text);
            Ok(())
        }
        fn sleep_ms(&mut self, _ms: u64) {}
    }

    #[derive(Default)]
    struct VirtualClock {
        now: u128,
    }
    impl Clock for VirtualClock {
        fn now_ms(&self) -> u128 {
            self.now
        }
        fn sleep_ms(&mut self, ms: u64) {
            self.now += ms as u128;
        }
    }

    fn engine() -> Engine<MockBrowser, MockBridge, CountingSink, InMemorySecrets, VirtualClock> {
        Engine::new(
            MockBrowser::default(),
            MockBridge::default(),
            UserClient::new(CountingSink::default(), 1),
            InMemorySecrets::new().with("APP_SECRET_PASSWORD", "s3cret"),
            VirtualClock::default(),
        )
    }

    fn page() -> PageContext {
        let json = r##"{
            "domain": "job-board.example",
            "schemaVersion": "1.0",
            "steps": [
                {"id":"go","action":{"type":"navigate","url":"https://job-board.example"}},
                {"id":"wait","action":{"type":"waitFor","condition":{"type":"elementVisible","selector":".job-list"}}},
                {"id":"login","action":{"type":"type","selector":"#password","value":"env:APP_SECRET_PASSWORD","isSecret":true}},
                {"id":"submit","action":{"type":"click","selector":"#submit"}},
                {"id":"title","action":{"type":"extract","selector":"h1","extractType":"text","saveToVariable":"page_title"}},
                {"id":"scrape","action":{"type":"extractCollection","containerSelector":".job-list","itemSelector":".job-card","extract":{"title":{"selector":"h2","extractType":"text"}},"saveToVariable":"jobs"}},
                {"id":"agg","action":{"type":"aggregateStrings","inputVariable":"jobs","template":"* {{title}} {{link}}","joinWith":"\n","saveToVariable":"body"}}
            ],
            "canvas": {"nodes":[{"stepId":"go","position":{"x":0,"y":0}}],"edges":[]}
        }"##;
        serde_json::from_str(json).unwrap()
    }

    #[test]
    fn runs_navigate_wait_type_click_extract_in_order() {
        let mut eng = engine();
        eng.run(&page()).unwrap();
        assert_eq!(eng.browser.log, vec!["navigate:https://job-board.example"]);
        assert_eq!(eng.user.sink_mut().typed, "s3cret");
        assert!(eng.user.sink_mut().clicks >= 2); // focus-click for type + submit click
        assert_eq!(
            eng.state.get("page_title").unwrap(),
            &serde_json::json!("Hello")
        );
    }

    #[test]
    fn aggregate_step_builds_joined_body() {
        let mut eng = engine();
        eng.run(&page()).unwrap();
        assert_eq!(
            eng.state.get("body").unwrap(),
            &serde_json::json!("* Job A http://a\n* Job B http://b")
        );
    }

    #[test]
    fn missing_secret_fails_validation_before_run() {
        let mut eng = Engine::new(
            MockBrowser::default(),
            MockBridge::default(),
            UserClient::new(CountingSink::default(), 1),
            InMemorySecrets::new(),
            VirtualClock::default(),
        );
        let err = eng.run(&page()).unwrap_err();
        assert!(matches!(err, EngineError::Step { .. }));
    }

    #[test]
    fn canvas_is_ignored_by_execution() {
        // page has a canvas with only one node but 7 steps; all steps still run.
        let mut eng = engine();
        eng.run(&page()).unwrap();
        assert!(eng.state.get("jobs").is_some());
    }
}
