mod node_ui;
mod egui_render_loop;
mod graph_processor;
mod graph;
mod node_update;
mod node_shader;
mod node_types;
mod spout_out_shader;
mod def;
// mod from_isf;
mod node_tree_ui;
mod prop_ui;

pub use graph_processor::ShaderGraphProcessor;
pub use egui_render_loop::render_glium;