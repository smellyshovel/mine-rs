//! The stopwatch implementation for the game.
//!
//! Has been taken from here (https://github.com/ellisonch/rust-stopwatch) and adopted for the needs
//! of the game.

use std::default::Default;
use std::time::{Duration, Instant};

#[derive(Debug, Default, Clone, Copy)]
pub struct Stopwatch {
    /// The time the stopwatch has been started last time (`None` if it's currently stopped or has
    /// never been started yet).
    start_time: Option<Instant>,
    /// The time elapsed while the stopwatch was running (between `start`s and `stop`s).
    elapsed: Duration,
}

impl Stopwatch {
    /// Starts the stopwatch.
    pub fn start(&mut self) {
        self.start_time = Some(Instant::now());
    }

    /// Stops the stopwatch.
    pub fn stop(&mut self) {
        self.elapsed = self.get_elapsed_time();
        self.start_time = None;
    }

    /// Returns the elapsed time since the first start of the stopwatch.
    pub fn get_elapsed_time(&self) -> Duration {
        let mut elapsed = self.elapsed;

        // if the stopwatch's running
        if let Some(start_time) = self.start_time {
            elapsed += start_time.elapsed()
        };

        elapsed
    }
}

#[cfg(test)]
mod test {
    use super::Stopwatch;
    use std::time::Duration;

    static SLEEP_MS: i64 = 50;
    static TOLERANCE_PERCENTAGE: f64 = 0.3;

    #[test]
    fn a_stopwatch_that_has_never_been_started_has_zero_as_the_elapsed_time_value() {
        let sw = Stopwatch::default();
        assert_eq!(sw.get_elapsed_time().as_millis(), 0);
    }

    #[cfg(not(target_os = "macos"))]
    #[test]
    fn the_stopwatch_correctly_measures_the_elapsed_time() {
        let mut sw = Stopwatch::default();
        sw.start();

        sleep_ms(SLEEP_MS);

        assert_sw_near(sw, SLEEP_MS);
    }

    #[test]
    fn repeated_toggling_does_not_affect_the_elapsed_time() {
        let mut sw = Stopwatch::default();
        sw.start();

        for _ in 0..1000 {
            sw.stop();
            sw.start();
        }

        assert_sw_near(sw, 0);
    }

    #[cfg(not(target_os = "macos"))]
    #[test]
    fn the_time_is_not_running_when_the_stopwatch_is_stopped() {
        let mut sw = Stopwatch::default();
        sw.start();

        sleep_ms(SLEEP_MS);

        sw.stop();
        assert_sw_near(sw, SLEEP_MS);

        sleep_ms(SLEEP_MS);

        assert_sw_near(sw, SLEEP_MS);
    }

    #[test]
    fn the_time_keeps_adding_after_a_stopwatch_gets_resumed() {
        let mut sw = Stopwatch::default();
        sw.start();

        sleep_ms(SLEEP_MS);

        sw.stop();
        assert_sw_near(sw, SLEEP_MS);

        sw.start();

        sleep_ms(SLEEP_MS);

        sw.stop();
        assert_sw_near(sw, 2 * SLEEP_MS);

        sw.start();

        sleep_ms(SLEEP_MS);

        assert_sw_near(sw, 3 * SLEEP_MS);
    }

    // helpers

    fn sleep_ms(ms: i64) {
        std::thread::sleep(Duration::from_millis(ms as u64))
    }

    fn assert_sw_near(sw: Stopwatch, elapsed: i64) {
        fn assert_near(x: i64, y: i64, tolerance: i64) {
            let diff = (x - y).abs();
            if diff > tolerance {
                panic!("Expected {:?}, got {:?}", x, y);
            }
        }

        let tolerance_value = (TOLERANCE_PERCENTAGE * elapsed as f64) as i64;

        assert_near(
            elapsed,
            sw.get_elapsed_time().as_millis() as i64,
            tolerance_value,
        );
    }
}
