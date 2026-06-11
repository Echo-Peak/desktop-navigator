use super::bridge::ScreenPoint;
use super::error::EngineResult;

#[derive(Debug, Clone)]
pub struct Rng {
    state: u64,
}

impl Rng {
    pub fn new(seed: u64) -> Self {
        Self {
            state: seed | 1,
        }
    }

    fn next_u64(&mut self) -> u64 {
        let mut x = self.state;
        x ^= x << 13;
        x ^= x >> 7;
        x ^= x << 17;
        self.state = x;
        x
    }

    pub fn next_f64(&mut self) -> f64 {
        (self.next_u64() >> 11) as f64 / (1u64 << 53) as f64
    }

    pub fn range(&mut self, lo: f64, hi: f64) -> f64 {
        lo + (hi - lo) * self.next_f64()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct Motion {
    pub points: Vec<ScreenPoint>,
    pub delays_ms: Vec<u64>,
}

impl Motion {
    pub fn total_delay_ms(&self) -> u64 {
        self.delays_ms.iter().sum()
    }
}

fn cubic_bezier(p0: (f64, f64), p1: (f64, f64), p2: (f64, f64), p3: (f64, f64), t: f64) -> (f64, f64) {
    let u = 1.0 - t;
    let w0 = u * u * u;
    let w1 = 3.0 * u * u * t;
    let w2 = 3.0 * u * t * t;
    let w3 = t * t * t;
    (
        w0 * p0.0 + w1 * p1.0 + w2 * p2.0 + w3 * p3.0,
        w0 * p0.1 + w1 * p1.1 + w2 * p2.1 + w3 * p3.1,
    )
}

pub fn plan_motion(start: ScreenPoint, end: ScreenPoint, rng: &mut Rng) -> Motion {
    let p0 = (start.x as f64, start.y as f64);
    let p3 = (end.x as f64, end.y as f64);
    let dx = p3.0 - p0.0;
    let dy = p3.1 - p0.1;
    let dist = (dx * dx + dy * dy).sqrt();

    if dist < 1.0 {
        return Motion {
            points: vec![start, end],
            delays_ms: vec![1, 1],
        };
    }

    let steps = ((dist / 6.0) as usize).clamp(16, 120);
    let nx = -dy / dist;
    let ny = dx / dist;
    let j1 = rng.range(-0.18, 0.18) * dist;
    let j2 = rng.range(-0.18, 0.18) * dist;
    let p1 = (p0.0 + dx * 0.3 + nx * j1, p0.1 + dy * 0.3 + ny * j1);
    let p2 = (p0.0 + dx * 0.7 + nx * j2, p0.1 + dy * 0.7 + ny * j2);

    let mut points = Vec::with_capacity(steps + 2);
    points.push(start);
    for i in 1..steps {
        let t = i as f64 / steps as f64;
        let (x, y) = cubic_bezier(p0, p1, p2, p3, t);
        points.push(ScreenPoint {
            x: x.round() as i32,
            y: y.round() as i32,
        });
    }

    let over = rng.range(2.0, 6.0);
    let overshoot = ScreenPoint {
        x: (p3.0 + dx / dist * over).round() as i32,
        y: (p3.1 + dy / dist * over).round() as i32,
    };
    points.push(overshoot);
    points.push(end);

    let total = (40.0 + dist * 1.2 * rng.range(0.85, 1.15)) as u64;
    let per = (total / points.len() as u64).max(1);
    let delays_ms = vec![per; points.len()];

    Motion { points, delays_ms }
}

pub trait InputSink {
    fn move_to(&mut self, x: i32, y: i32) -> EngineResult<()>;
    fn left_click(&mut self) -> EngineResult<()>;
    fn double_click(&mut self) -> EngineResult<()>;
    fn right_click(&mut self) -> EngineResult<()>;
    fn scroll(&mut self, delta_x: i32, delta_y: i32) -> EngineResult<()>;
    fn key_combo(&mut self, keys: &[String]) -> EngineResult<()>;
    fn type_text(&mut self, text: &str) -> EngineResult<()>;
    fn sleep_ms(&mut self, ms: u64);
}

pub struct UserClient<S: InputSink> {
    sink: S,
    pos: ScreenPoint,
    rng: Rng,
}

impl<S: InputSink> UserClient<S> {
    pub fn new(sink: S, seed: u64) -> Self {
        Self {
            sink,
            pos: ScreenPoint { x: 0, y: 0 },
            rng: Rng::new(seed),
        }
    }

    pub fn position(&self) -> ScreenPoint {
        self.pos
    }

    pub fn sink_mut(&mut self) -> &mut S {
        &mut self.sink
    }

    pub fn into_sink(self) -> S {
        self.sink
    }

    pub fn move_to(&mut self, target: ScreenPoint) -> EngineResult<()> {
        let motion = plan_motion(self.pos, target, &mut self.rng);
        for (point, delay) in motion.points.iter().zip(motion.delays_ms.iter()) {
            self.sink.move_to(point.x, point.y)?;
            self.sink.sleep_ms(*delay);
            self.pos = *point;
        }
        self.pos = target;
        Ok(())
    }

    pub fn click(&mut self, target: ScreenPoint) -> EngineResult<()> {
        self.move_to(target)?;
        self.sink.sleep_ms(self.rng.range(30.0, 90.0) as u64);
        self.sink.left_click()
    }

    pub fn double_click(&mut self, target: ScreenPoint) -> EngineResult<()> {
        self.move_to(target)?;
        self.sink.double_click()
    }

    pub fn right_click(&mut self, target: ScreenPoint) -> EngineResult<()> {
        self.move_to(target)?;
        self.sink.right_click()
    }

    pub fn scroll(&mut self, delta_x: i32, delta_y: i32) -> EngineResult<()> {
        self.sink.scroll(delta_x, delta_y)
    }

    pub fn key_combo(&mut self, keys: &[String]) -> EngineResult<()> {
        self.sink.key_combo(keys)
    }

    pub fn type_text(&mut self, text: &str) -> EngineResult<()> {
        for ch in text.chars() {
            self.sink.type_text(&ch.to_string())?;
            self.sink.sleep_ms(self.rng.range(50.0, 120.0) as u64);
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Default)]
    struct RecordingSink {
        moves: Vec<(i32, i32)>,
        clicks: usize,
        slept: u64,
        typed: String,
    }

    impl InputSink for RecordingSink {
        fn move_to(&mut self, x: i32, y: i32) -> EngineResult<()> {
            self.moves.push((x, y));
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
        fn sleep_ms(&mut self, ms: u64) {
            self.slept += ms;
        }
    }

    #[test]
    fn move_never_teleports_starts_at_current_ends_at_target() {
        let mut client = UserClient::new(RecordingSink::default(), 42);
        let target = ScreenPoint { x: 500, y: 300 };
        client.move_to(target).unwrap();
        let sink = client.into_sink();
        assert!(sink.moves.len() > 2, "should travel via many waypoints");
        assert_eq!(sink.moves.first().copied(), Some((0, 0)));
        assert_eq!(sink.moves.last().copied(), Some((500, 300)));
        assert!(sink.slept > 0, "travel must take nonzero time");
    }

    #[test]
    fn cursor_position_is_tracked_between_moves() {
        let mut client = UserClient::new(RecordingSink::default(), 7);
        client.move_to(ScreenPoint { x: 100, y: 100 }).unwrap();
        assert_eq!(client.position(), ScreenPoint { x: 100, y: 100 });
        client.move_to(ScreenPoint { x: 250, y: 80 }).unwrap();
        assert_eq!(client.position(), ScreenPoint { x: 250, y: 80 });
        let sink = client.into_sink();
        // second leg must begin from the previously tracked position
        let first_leg_end = (100, 100);
        let idx = sink.moves.iter().position(|p| *p == first_leg_end).unwrap();
        assert_eq!(sink.moves[idx], first_leg_end);
    }

    #[test]
    fn travel_duration_scales_with_distance() {
        let short = plan_motion(
            ScreenPoint { x: 0, y: 0 },
            ScreenPoint { x: 20, y: 0 },
            &mut Rng::new(1),
        );
        let long = plan_motion(
            ScreenPoint { x: 0, y: 0 },
            ScreenPoint { x: 1200, y: 0 },
            &mut Rng::new(1),
        );
        assert!(long.points.len() > short.points.len());
        assert!(long.total_delay_ms() > short.total_delay_ms());
    }

    #[test]
    fn click_moves_then_clicks() {
        let mut client = UserClient::new(RecordingSink::default(), 3);
        client.click(ScreenPoint { x: 60, y: 60 }).unwrap();
        let sink = client.into_sink();
        assert_eq!(sink.clicks, 1);
        assert_eq!(sink.moves.last().copied(), Some((60, 60)));
    }

    #[test]
    fn type_text_emits_each_char() {
        let mut client = UserClient::new(RecordingSink::default(), 9);
        client.type_text("hi").unwrap();
        assert_eq!(client.into_sink().typed, "hi");
    }
}
