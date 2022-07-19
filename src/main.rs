// mod renderer_nannou_view;

use clap::Parser;
use glium_rpi::{self, ModelOptions};
use tracing::metadata::LevelFilter;
use tracing_subscriber::fmt;

//pretty stars
#[derive(Parser, Debug)]
#[clap(author, version, about, long_about = None)]
struct Args {
    /// Number of stars to render
    #[clap(short, long, value_parser, default_value_t = 1000)]
    num_stars: usize,

    #[clap(short, long, value_parser, default_value_t = 10.0)]
    speed: f32,
}

fn main() {
    let args = Args::parse();

    tracing_subscriber::fmt()
        .with_max_level(LevelFilter::TRACE)
        .init();

    glium_rpi::main(glium_rpi::Options {
        num_stars: args.num_stars,
        model_options: ModelOptions { speed: args.speed },
    });
}
