use std::{f32::consts::FRAC_PI_2, time::{SystemTime, Instant}};


// mod renderer_nannou_view;
mod nannou;
mod stars;

fn main() {
    let _args = std::env::args().skip(1);

    // println!("Hello, world! {}", args.next().unwrap());
    // main_kiss()
    nannou::main();
}