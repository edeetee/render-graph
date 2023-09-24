// mod renderer_nannou_view;
mod common;
mod fullscreen_shader;
mod gl_expression;
mod graph;
mod isf;
mod obj_shader;
mod textures;
mod tree_view;
pub mod util;
mod widgets;

use color_eyre::eyre::Result;

#[macro_use]
extern crate partial_application;

#[cfg(feature = "editor")]
mod editor;

#[cfg(feature = "webui")]
mod web_ui;

#[cfg(feature = "vst_plugin")]
mod vst_plugin;

mod egui_glium;

// use graph::render_glium;

fn main() -> Result<()> {
    color_eyre::install()?;

    egui_glium::main();

    Ok(())
}
