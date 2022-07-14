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

pub struct Model {
    pub stars: Stars,
    pub mat: Mat4,
    receiver: Receiver,
    options: Options,
    perf_record: PerformanceRecord
}

const PORT: u16 = 10000;
const PERF_UPDATE_DURATION: Duration = Duration::from_secs(2);

pub struct UpdateInfo {
    pub time_since_previous: Duration,
    pub frames_since_previous: u32
}

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
            perf_record: PerformanceRecord::default()
        }
    }

    pub fn update(&mut self, info: UpdateInfo){
        self.stars.update(info.time_since_previous.as_secs_f32()*self.options.speed);

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
        
        self.perf_record.frames_count += info.frames_since_previous;
        let now = Instant::now();

        if PERF_UPDATE_DURATION < (now - self.perf_record.time_started){

            if let Some(duration) = self.perf_record.avg_frame_period(now){
                let frame_s = duration.as_secs_f32();
                let fps = 1.0/frame_s;
                let frame_ms = frame_s*1000.;
            
                println!("{frame_ms:.1}ms, {fps:.1}fps");
    
                self.perf_record = PerformanceRecord {
                    frames_count: 0,
                    time_started: now
                };
            }
        }
    }
}

fn parse_float<'a>(args: Vec<OscType>) -> Option<f32> {
    if let OscType::Float(result) = args.first()?{
        Some(result.clone())
    } else {
        None
    }
}