use egui_glium::EguiGlium;

use super::ShaderGraphProcessor;


pub struct Renderer {
    processor: ShaderGraphProcessor
}

impl Renderer {
    pub fn new() -> Self {

        let mut egui_glium = EguiGlium::new(&display, &event_loop);

        let mut shader_node_graph = ShaderGraphProcessor::load_from_file_or_default();
        Self {
            processor: shader_node_graph
        }
    }

    pub fn draw(&mut self) {

    }

    pub fn update(&mut self) {

    }
}