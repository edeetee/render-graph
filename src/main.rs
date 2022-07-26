// mod renderer_nannou_view;

use clap::Parser;
use tracing::metadata::LevelFilter;

mod stars;
use stars::*;

mod graph;

mod render_egui;

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
    
    let native_options = eframe::NativeOptions::default();
    eframe::run_native("Shader Graph", native_options, 
    Box::new(|cc| 
        Box::new(graph::NodeGraphApp::new(cc))
    ));

    render_stars(Options {
        num_stars: args.num_stars,
        model_options: ModelOptions { speed: args.speed },
    });
}
