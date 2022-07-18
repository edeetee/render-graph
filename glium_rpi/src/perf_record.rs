struct PerformanceRecord {
    frames_count: u32,
    time_started: Instant
}

impl Default for PerformanceRecord{
    fn default() -> Self {
        Self { frames_count: Default::default(), time_started: Instant::now() }
    }
}

impl PerformanceRecord {
    fn avg_frame_period(&self, now: Instant) -> Option<Duration> {
        let time_elapsed = now - self.time_started;
        time_elapsed.checked_div(self.frames_count as _)
    }
}