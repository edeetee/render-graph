

///To be used with 
// pub struct PerformanceRecord<T> {
//     value: T,
//     time_started: Instant
// }

// impl Default for PerformanceRecord{
//     fn default() -> Self {
//         Self { frames_count: Default::default(), time_started: Instant::now() }
//     }
// }

// impl PerformanceRecord {
//     fn avg_frame_period(&self, now: Instant) -> Option<Duration> {
//         let time_elapsed = now - self.time_started;
//         time_elapsed.checked_div(self.frames_count as _)
//     }
// }

// fn perf(){
//     self.perf_record.frames_count += info.frames_since_previous;
//     let now = Instant::now();
    
//     if PERF_UPDATE_DURATION < (now - self.perf_record.time_started){
    
//         if let Some(duration) = self.perf_record.avg_frame_period(now){
//             let frame_s = duration.as_secs_f32();
//             let fps = 1.0/frame_s;
//             let frame_ms = frame_s*1000.;
        
//             println!("{frame_ms:.1}ms, {fps:.1}fps");
    
//             self.perf_record = PerformanceRecord {
//                 frames_count: 0,
//                 time_started: now
//             };
//         }
//     }
// }
        