// mod renderer_nannou_view;

use clap::Parser;
use glium_rpi::{self, ModelOptions};

//pretty stars
#[derive(Parser, Debug)]
#[clap(author, version, about, long_about = None)]
struct Args {
    /// Number of stars to render
    #[clap(short, long, value_parser, default_value_t=1000)]
    num_stars: usize,

    #[clap(short, long, value_parser, default_value_t=10.0)]
    speed: f32
}

fn main() {
    let args = Args::parse();
    // main_kiss()
    // nannou::main();
    // glium::main();
    // let guard = pprof::ProfilerGuardBuilder::default().frequency(1000).blocklist(&["libc", "libgcc", "pthread", "vdso"]).build().unwrap();

    glium_rpi::main(glium_rpi::Options {
        num_stars: args.num_stars,
        model_options: ModelOptions {
            speed: args.speed
        }
    });

    // if let Ok(report) = guard.report().build() {
    //     let file = File::create("flamegraph.svg").unwrap();
    //     report.flamegraph(file).unwrap();
    // };
}