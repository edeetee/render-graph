// mod renderer_nannou_view;
mod graph;
mod fullscreen_shader;
mod textures;
mod isf;
mod tree_view;
mod obj_shader;
pub mod util;
mod gl_expression;
mod common;
use color_eyre::eyre::Result;

#[macro_use]
extern crate partial_application;

#[cfg(feature="editor")]
mod editor;
// mod egui_render_loop;

#[cfg(feature="webui")]
mod web_ui;

#[cfg(feature="vst_plugin")]
mod vst_plugin;

mod egui_glium;

// use graph::render_glium;

fn main() -> Result<()> {

    color_eyre::install()?;

    egui_glium::main();

    Ok(())
}