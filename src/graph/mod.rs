mod node_ui;
mod graph_processor;
mod node_update;
mod node_shader;
mod node_types;
mod spout_out_shader;
// mod from_isf;
mod node_tree_ui;
mod prop_ui;
mod graph_ui;

pub(crate) mod graph;
pub(crate) mod def;

pub use graph_processor::ShaderGraphProcessor;
pub use graph_ui::GraphUi;
// pub use egui_render_loop::render_glium;