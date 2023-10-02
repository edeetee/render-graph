// mod renderer_nannou_view;
mod common;
mod fullscreen_shader;
mod gl_expression;
mod graph;
mod isf;
mod obj_shader;
mod textures;
pub mod util;

use color_eyre::eyre::Result;

#[macro_use]
extern crate partial_application;

#[cfg(feature = "editor")]
mod editor;
#[cfg(feature = "editor")]
mod tree_view;
#[cfg(feature = "editor")]
mod widgets;

#[cfg(feature = "webui")]
mod web_ui;

#[cfg(feature = "vst_plugin")]
mod vst_plugin;

#[cfg(feature = "editor")]
mod egui_glium;

// use graph::render_glium;

fn main() -> Result<()> {
    color_eyre::install()?;

    #[cfg(feature = "editor")]
    egui_glium::main();

    Ok(())
}
