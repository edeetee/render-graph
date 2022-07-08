use nannou::wgpu::Backends;
use nannou::{prelude::*};
use nannou_osc as osc;
use nannou_osc::rosc::OscType;
use osc::Receiver;
use nannou::winit::window::Fullscreen;
use clap::Parser;

use crate::stars::Stars;

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

struct NannouState {
    stars: Stars,
    mat: Mat4,
    receiver: Receiver,
    options: Options,
    last_elapsed_frames: u64
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

    let elapsed_frames = app.elapsed_frames();
    let frames_since_last_update = elapsed_frames - state.last_elapsed_frames;

    let frame_ms = update.since_last.as_secs_f32()/frames_since_last_update.to_f32().unwrap()*1000.0;

    println!("{frame_ms:.0}ms per frame");

    state.last_elapsed_frames = elapsed_frames

    // update.since_last
}

const PORT: u16 = 10000;

fn start_nannou(app: &App) -> NannouState {
    let args = Args::parse();

    let _window_id = app
        .new_window()
        .title("Stars")
        .view(view_nannou)
        .fullscreen_with(Some(Fullscreen::Borderless(app.primary_monitor())))
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

    let receiver = osc::receiver(PORT).unwrap();

    let options = Options{
        speed: 1.
    };
 
    NannouState { stars, mat: perspective, receiver, options, last_elapsed_frames: 0 }
}

fn parse_float<'a>(args: Vec<OscType>) -> Option<f32> {
    if let OscType::Float(result) = args.first()?{
        Some(result.clone())
    } else {
        None
    }
}