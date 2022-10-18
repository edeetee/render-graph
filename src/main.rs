// mod renderer_nannou_view;
mod graph;
mod fullscreen_shader;
mod textures;
mod isf;
mod tree_view;
mod obj_shader;
mod util;
mod gl_expression;

use graph::render_glium;

fn main() {
    render_glium();
}
