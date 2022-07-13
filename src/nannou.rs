use std::time::{Instant, Duration};

use nannou::wgpu::Backends;
use nannou::{prelude::*};
use nannou_osc as osc;
use nannou_osc::rosc::OscType;
use osc::Receiver;
use nannou::winit::window::Fullscreen;
use clap::Parser;

//pretty stars
#[derive(Parser, Debug)]
#[clap(author, version, about, long_about = None)]
struct Args {
    /// Number of stars to render
    #[clap(short, long, value_parser, default_value_t=10)]
    num_stars: usize,
}

const BACKEND: Backends = if cfg!(target_os="linux"){
    Backends::GL
} else {
    Backends::PRIMARY
};
// const BACKEND: Backends = Backends::PRIMARY;

pub fn main() {
    println!("Using {BACKEND:?} backend");

    nannou::app(start_nannou)
        .update(update_nannou)
        .backends(BACKEND)
        .run()
}

struct Options{
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

impl PerformanceSnapshot {
    fn avg_frame_period(&self, prev: &Self) -> Duration {
        let frames_elapsed = self.total_frames - prev.total_frames;
        let time_elapsed = self.time_captured - prev.time_captured;

        time_elapsed.div_f32(frames_elapsed.to_f32().unwrap())
    }
}

struct NannouState {
    stars: Stars,
    mat: Mat4,
    receiver: Receiver,
    options: Options,
    last_perf_info: PerformanceSnapshot
}

fn view_nannou(app: &App, model: &NannouState, frame: Frame){
    frame.clear(BLACK);

    let draw = app.draw()
        .transform(model.mat);

    for star in model.stars.iter() {
        let closeness = 0.995f32.pow(star.pos.length());
        draw
            .xyz(star.pos)
            .ellipse()
            .radius(star.radius*closeness)
            .color(star.color);
    }

    draw.to_frame(app, &frame).unwrap()
}

const PERF_UPDATE_DURATION: Duration = Duration::from_secs(2);

fn update_nannou(app: &App, state: &mut NannouState, update: Update){
    state.stars.update(update.since_last.as_secs_f32()*state.options.speed);

    for (packet, _) in state.receiver.try_iter(){
        for msg in packet.into_msgs(){
            match msg.addr.as_str() {
                "/speed" => {
                    if let Some(speed) = msg.args.map(parse_float).flatten(){
                        state.options.speed = speed;
                        println!("osc speed: {speed:.2}");
                    }
                }
                _ => {}
            }
        }
    }

    let now = Instant::now();
    if PERF_UPDATE_DURATION < (now - state.last_perf_info.time_captured){

        let perf_snapshot = PerformanceSnapshot{
            total_frames: app.elapsed_frames(),
            time_captured: now
        };
        let frame_s = perf_snapshot.avg_frame_period(&state.last_perf_info).as_secs_f32();
        let fps = frame_s.inv();
        let frame_ms = frame_s*1000.;
    
        println!("{frame_ms:.0}ms, {fps:.0}fps");
        state.last_perf_info = perf_snapshot;
    }

}

const PORT: u16 = 10000;

fn start_nannou(app: &App) -> NannouState {
    let args = Args::parse();

    // let video_mode = app.primary_monitor()
    //     .and_then(|monitor| monitor.video_modes().next())
    //     .expect("No video mode found");
    // let fullscreen_mode = Some(Fullscreen::Exclusive(video_mode));

    let fullscreen_mode = Some(Fullscreen::Borderless(None));

    let _window_id = app
        .new_window()
        .title("Stars")
        .view(view_nannou)
        .fullscreen_with(fullscreen_mode)
        .build()
        .unwrap();

    // let window = app.window(window_id).unwrap();
    // let (w, h) = window.rect().w_h();
    // let window_scale = Vec3::new(w, h, 1.);
    // let scale = Mat4::from_scale(window_scale);
    
    let stars = Stars::new(args.num_stars);
    let perspective = Mat4::perspective_rh(
        std::f32::consts::FRAC_PI_8, 
        1., 
        0.001, 
        1000.
    );
 
    NannouState { 
        stars, 
        mat: perspective, 

        receiver: osc::receiver(PORT).unwrap(),

        options: Options::default(),
        last_perf_info: PerformanceSnapshot::default()
    }
}

fn parse_float<'a>(args: Vec<OscType>) -> Option<f32> {
    if let OscType::Float(result) = args.first()?{
        Some(result.clone())
    } else {
        None
    }
}