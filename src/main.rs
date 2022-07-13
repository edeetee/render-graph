// mod renderer_nannou_view;

use clap::Parser;
// mod nannou;
use glium_rpi::{self, Options};

//pretty stars
#[derive(Parser, Debug)]
#[clap(author, version, about, long_about = None)]
struct Args {
    /// Number of stars to render
    #[clap(short, long, value_parser, default_value_t=10)]
    num_stars: usize,
}

fn main() {
    let args = Args::parse();
    // main_kiss()
    // nannou::main();
    // glium::main();
    glium_rpi::main(Options {
        num_stars: args.num_stars
    });
}