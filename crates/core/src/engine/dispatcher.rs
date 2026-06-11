use serde_json::Value;

use crate::schema::{AutomationAction, AutomationStep, PageContext, WaitCondition};

use super::bridge::{rect_to_screen, DomBridge, ScreenPoint};
use super::browser::BrowserManager;
use super::error::{EngineError, EngineResult};
use super::input::{InputSink, UserClient};
use super::secret::{self, SecretResolver};
use super::state::StateMap;

pub trait Clock {
    fn now_ms(&self) -> u128;
    fn sleep_ms(&mut self, ms: u64);
}

pub struct SystemClock;

impl Clock for SystemClock {
    fn now_ms(&self) -> u128 {
        crate::data::now_ms()
    }
    fn sleep_ms(&mut self, ms: u64) {
        std::thread::sleep(std::time::Duration::from_millis(ms));
    }
}

const DEFAULT_WAIT_MS: u64 = 10_000;
const POLL_INTERVAL_MS: u64 = 50;

pub struct Engine<B, D, S, R, C>
where
    B: BrowserManager,
    D: DomBridge,
    S: InputSink,
    R: SecretResolver,
    C: Clock,
{
    pub browser: B,
    pub bridge: D,
    pub user: UserClient<S>,
    pub secrets: R,
    pub clock: C,
    pub state: StateMap,
}

impl<B, D, S, R, C> Engine<B, D, S, R, C>
where
    B: BrowserManager,
    D: DomBridge,
    S: InputSink,
    R: SecretResolver,
    C: Clock,
{
    pub fn new(browser: B, bridge: D, user: UserClient<S>, secrets: R, clock: C) -> Self {
        Self {
            browser,
            bridge,
            user,
            secrets,
            clock,
            state: StateMap::new(),
        }
    }

    pub fn validate(&self, page: &PageContext) -> EngineResult<()> {
        let mut seen = std::collections::HashSet::new();
        for step in &page.steps {
            if !seen.insert(step.id.as_str()) {
                return Err(EngineError::Validation(format!(
                    "duplicate step id '{}'",
                    step.id
                )));
            }
            if let AutomationAction::Type {
                value, is_secret, ..
            } = &step.action
            {
                if *is_secret || secret::is_secret_ref(value) {
                    secret::resolve_value(value, &self.secrets).map_err(|e| EngineError::Step {
                        id: step.id.clone(),
                        source: Box::new(e),
                    })?;
                }
            }
        }
        Ok(())
    }

    pub fn run(&mut self, page: &PageContext) -> EngineResult<()> {
        self.validate(page)?;
        self.state.clear();
        for step in &page.steps {
            match self.run_step(step) {
                Ok(()) => {}
                Err(err) => {
                    if step.optional.unwrap_or(false) {
                        continue;
                    }
                    return Err(EngineError::Step {
                        id: step.id.clone(),
                        source: Box::new(err),
                    });
                }
            }
        }
        Ok(())
    }

    fn run_step(&mut self, step: &AutomationStep) -> EngineResult<()> {
        let attempts = step.retries.unwrap_or(0) + 1;
        let mut last_err = None;
        for _ in 0..attempts {
            match self.execute(&step.action, step.timeout_ms) {
                Ok(()) => return Ok(()),
                Err(e) => last_err = Some(e),
            }
        }
        Err(last_err.unwrap_or_else(|| EngineError::Input("no attempts run".into())))
    }

    fn execute(&mut self, action: &AutomationAction, timeout_ms: Option<u64>) -> EngineResult<()> {
        match action {
            AutomationAction::Navigate { url } => {
                let url = self.state.render_template(url);
                self.browser.navigate(&url)
            }
            AutomationAction::Reload => self.browser.reload(),
            AutomationAction::GoBack => self.browser.go_back(),
            AutomationAction::GoForward => self.browser.go_forward(),

            AutomationAction::MoveMouse { x, y, .. } => self.user.move_to(ScreenPoint {
                x: x.round() as i32,
                y: y.round() as i32,
            }),
            AutomationAction::Click { selector, offset } => {
                let point = self.screen_point(selector, offset.as_ref())?;
                self.user.click(point)
            }
            AutomationAction::DoubleClick { selector, offset } => {
                let point = self.screen_point(selector, offset.as_ref())?;
                self.user.double_click(point)
            }
            AutomationAction::RightClick { selector, offset } => {
                let point = self.screen_point(selector, offset.as_ref())?;
                self.user.right_click(point)
            }
            AutomationAction::Type {
                selector,
                value,
                is_secret: _,
            } => {
                let point = self.screen_point(selector, None)?;
                self.user.click(point)?;
                let resolved = secret::resolve_value(value, &self.secrets)?;
                self.user.type_text(&resolved)
            }
            AutomationAction::KeyboardShortcut { keys } => self.user.key_combo(keys),
            AutomationAction::Scroll {
                delta_x, delta_y, ..
            } => self.user.scroll(*delta_x as i32, *delta_y as i32),

            AutomationAction::WaitFor {
                condition,
                skip_after,
            } => self.wait_for(condition, skip_after.or(timeout_ms)),
            AutomationAction::Extract {
                selector,
                extract_type,
                attribute_name,
                save_to_variable,
            } => {
                let value = match extract_type {
                    crate::schema::ExtractType::Text => {
                        Value::String(self.bridge.get_text(selector)?)
                    }
                    crate::schema::ExtractType::Attribute => {
                        let attr = attribute_name.as_deref().ok_or_else(|| {
                            EngineError::Validation("extract attribute requires attributeName".into())
                        })?;
                        Value::String(self.bridge.get_attribute(selector, attr)?)
                    }
                };
                self.state.set(save_to_variable.clone(), value);
                Ok(())
            }
            AutomationAction::ExtractCollection {
                container_selector,
                item_selector,
                extract,
                save_to_variable,
            } => {
                let value =
                    self.bridge
                        .extract_collection(container_selector, item_selector, extract)?;
                self.state.set(save_to_variable.clone(), value);
                Ok(())
            }
            AutomationAction::QueryProperty {
                target,
                selector,
                property,
                save_to_variable,
            } => {
                let value =
                    self.bridge
                        .query_property(*target, selector.as_deref(), property)?;
                self.state.set(save_to_variable.clone(), value);
                Ok(())
            }
            AutomationAction::AggregateStrings {
                input_variable,
                template,
                join_with,
                save_to_variable,
            } => {
                let result = self.state.aggregate(input_variable, template, join_with)?;
                self.state.set(save_to_variable.clone(), Value::String(result));
                Ok(())
            }
        }
    }

    fn screen_point(
        &mut self,
        selector: &str,
        offset: Option<&crate::schema::Offset>,
    ) -> EngineResult<ScreenPoint> {
        let (rect, window) = self.bridge.element_geometry(selector)?;
        Ok(rect_to_screen(&rect, &window, offset))
    }

    fn wait_for(&mut self, condition: &WaitCondition, skip_after: Option<u64>) -> EngineResult<()> {
        match condition {
            WaitCondition::Time { ms } => {
                self.clock.sleep_ms(*ms);
                Ok(())
            }
            WaitCondition::ElementVisible { selector } => {
                self.poll(skip_after, |b| b.is_visible(selector), true, selector)
            }
            WaitCondition::ElementHidden { selector } => {
                self.poll(skip_after, |b| b.is_visible(selector), false, selector)
            }
        }
    }

    fn poll(
        &mut self,
        skip_after: Option<u64>,
        check: impl Fn(&mut D) -> EngineResult<bool>,
        want_visible: bool,
        selector: &str,
    ) -> EngineResult<()> {
        let deadline = skip_after.unwrap_or(DEFAULT_WAIT_MS);
        let start = self.clock.now_ms();
        loop {
            if check(&mut self.bridge)? == want_visible {
                return Ok(());
            }
            if self.clock.now_ms().saturating_sub(start) >= deadline as u128 {
                return Err(EngineError::Timeout(format!(
                    "condition for '{selector}' not met within {deadline}ms"
                )));
            }
            self.clock.sleep_ms(POLL_INTERVAL_MS);
        }
    }
}

pub fn integration_payload(template: &Value, state: &StateMap) -> Value {
    match template {
        Value::String(s) => Value::String(state.render_template(s)),
        Value::Array(items) => {
            Value::Array(items.iter().map(|v| integration_payload(v, state)).collect())
        }
        Value::Object(map) => {
            let mut out = serde_json::Map::new();
            for (k, v) in map {
                out.insert(k.clone(), integration_payload(v, state));
            }
            Value::Object(out)
        }
        other => other.clone(),
    }
}
