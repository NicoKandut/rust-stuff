use logging::{log, LOG_RENDER};
use std::time::{Duration, Instant};

pub(crate) struct Stats {
    start: Instant,
    frames_this_second: usize,
    frames_total: usize,
}

impl Stats {
    pub(crate) fn new() -> Self {
        Self {
            start: Instant::now(),
            frames_this_second: 0,
            frames_total: 0,
        }
    }

    pub(crate) fn frames(&self) -> usize {
        self.frames_total
    }

    pub(crate) fn add_frame(&mut self) {
        self.frames_this_second += 1;
        self.frames_total += 1;
    }

    fn reset(&mut self) {
        self.start = Instant::now();
        self.frames_this_second = 0;
    }

    pub(crate) fn log_interval(&mut self, interval: &Duration) {
        if self.start.elapsed() > *interval {
            log!(*LOG_RENDER, "Stats: {} frames, ", self.frames_this_second);

            self.reset();
        }
    }
}
