use std::time::{Instant, Duration};

use glam::Mat4;
use nannou_osc as osc;
use nannou_osc::rosc::OscType;
use osc::Receiver;
use stars::Stars;

struct Options {
    speed: f32
}

impl Default for Options{
    fn default() -> Self {
        Self { speed: 1. }
    }
}

struct PerformanceSnapshot {
    total_frames: u64,
    time_captured: Instant
}

impl Default for PerformanceSnapshot{
    fn default() -> Self {
        Self { total_frames: Default::default(), time_captured: Instant::now() }
    }
}

// impl PerformanceSnapshot {
//     fn avg_frame_period(&self, prev: &Self) -> Duration {
//         let frames_elapsed = self.total_frames - prev.total_frames;
//         let time_elapsed = self.time_captured - prev.time_captured;

//         time_elapsed.div_f32(frames_elapsed.to_f32().unwrap())
//     }
// }

pub struct Model {
    pub stars: Stars,
    pub mat: Mat4,
    receiver: Receiver,
    options: Options,
    last_perf_info: PerformanceSnapshot
}

const PORT: u16 = 10000;
const PERF_UPDATE_DURATION: Duration = Duration::from_secs(2);

impl Model {
    pub fn new(num_stars: usize) -> Self {
        
        let stars = Stars::new(num_stars);
        let perspective = Mat4::perspective_rh(
            std::f32::consts::FRAC_PI_2, 
            1., 
            0.001, 
            1000.
        );
     
        Self { 
            stars, 
            mat: perspective, 
    
            receiver: osc::receiver(PORT).unwrap(),
    
            options: Options::default(),
            last_perf_info: PerformanceSnapshot::default()
        }
    }

    pub fn update(&mut self, seconds: f32){
        self.stars.update(seconds*self.options.speed);

        for (packet, _) in self.receiver.try_iter(){
            for msg in packet.into_msgs(){
                match msg.addr.as_str() {
                    "/speed" => {
                        if let Some(speed) = msg.args.map(parse_float).flatten(){
                            self.options.speed = speed;
                            println!("osc speed: {speed:.2}");
                        }
                    }
                    _ => {}
                }
            }
        }
    
        // let now = Instant::now();
        // if PERF_UPDATE_DURATION < (now - state.last_perf_info.time_captured){
    
        //     let perf_snapshot = PerformanceSnapshot{
        //         total_frames: app.elapsed_frames(),
        //         time_captured: now
        //     };
        //     let frame_s = perf_snapshot.avg_frame_period(&state.last_perf_info).as_secs_f32();
        //     let fps = frame_s.inv();
        //     let frame_ms = frame_s*1000.;
        
        //     println!("{frame_ms:.0}ms, {fps:.0}fps");
        //     self.last_perf_info = perf_snapshot;
        // }
    }
}

fn parse_float<'a>(args: Vec<OscType>) -> Option<f32> {
    if let OscType::Float(result) = args.first()?{
        Some(result.clone())
    } else {
        None
    }
}