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
    #[clap(short, long, value_parser)]
    num_stars: usize,
}

pub fn main() {
    nannou::app(start_nannou).update(update_nannou).run()
}

struct Options{
    speed: f32
}

struct NannouState {
    stars: Stars,
    mat: Mat4,
    receiver: Receiver,
    options: Options
}

fn view_nannou(app: &App, model: &NannouState, frame: Frame){
    frame.clear(BLACK);

    let draw = app.draw()
        .transform(model.mat);

    for star in model.stars.iter() {
        let closeness = 0.995f32.pow(star.pos.length());
        draw.xyz(star.pos).ellipse().radius(star.radius*closeness);
    }

    draw.to_frame(app, &frame).unwrap()
}

fn update_nannou(_app: &App, state: &mut NannouState, frame: Update){
    state.stars.update(frame.since_last.as_secs_f32()*state.options.speed);

    // Vec::new().first()

    for (packet, _) in state.receiver.try_iter(){
        for msg in packet.into_msgs(){
            match msg.addr.as_str() {
                "/speed" => {
                    if let Some(speed) = msg.args.map(parse_float).flatten(){
                        state.options.speed = speed;
                        println!("osc speed: {}", speed);
                    }
                }
                _ => {}
            }
        }
    }
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
 
    NannouState { stars, mat: perspective, receiver, options }
}

fn parse_float<'a>(args: Vec<OscType>) -> Option<f32> {
    if let OscType::Float(result) = args.first()?{
        Some(result.clone())
    } else {
        None
    }
}