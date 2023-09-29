use logging::{log, LOG_RENDER};
use std::time::{Duration, Instant};

pub(crate) struct Stats {
    start: Instant,
    frames: usize,
}

impl Stats {
    pub(crate) fn new() -> Self {
        Self {
            start: Instant::now(),
            frames: 0,
        }
    }

    pub(crate) fn add_frame(&mut self) {
        self.frames += 1;
    }

    fn reset(&mut self) {
        self.start = Instant::now();
        self.frames = 0;
    }

    pub(crate) fn log_interval(&mut self, interval: &Duration) {
        if self.start.elapsed() > *interval {
            log!(*LOG_RENDER, "Stats: {} frames, ", self.frames);

            self.reset();
        }
    }
}
