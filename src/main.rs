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
// mod egui_render_loop;

#[cfg(feature="vst_plugin")]
mod vst_plugin;

mod egui_render_loop;

use graph::render_glium;

fn main() {
    render_glium();
}
