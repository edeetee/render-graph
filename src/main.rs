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

#[cfg(feature="editor")]
mod editor;
// mod egui_render_loop;

#[cfg(feature="vst_plugin")]
mod vst_plugin;

mod egui_glium;

// use graph::render_glium;

fn main() {
    egui_glium::main();
}