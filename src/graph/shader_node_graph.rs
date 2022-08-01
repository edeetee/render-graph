use egui_glium::EguiGlium;
use egui_node_graph::{NodeId, GraphEditorState, NodeResponse};
use glium::{Frame, backend::Facade, Display, Surface};
use slotmap::{SecondaryMap};

use super::{def::{*, self}, trait_impl::AllNodeTypes, shader_manager::{ShaderData, new_shader_data}};

type EditorState = GraphEditorState<NodeData, NodeConnectionTypes, NodeValueTypes, NodeTypes, GraphState>;

pub struct ShaderNodeGraph
{
    pub graph_state: EditorState,
    output_nodes: Vec<NodeId>,
    shaders: SecondaryMap<NodeId, ShaderData<Frame>>
}

impl ShaderNodeGraph {
    pub fn new() -> Self {
        Self { 
            graph_state: GraphEditorState::new(1.0, GraphState::default()),
            output_nodes: Vec::new(),
            shaders: SecondaryMap::new()
        }
    }

    pub fn node_event(&mut self, display: &impl Facade, egui_glium: &mut EguiGlium, event: NodeResponse<def::GraphResponse, NodeData>) {
        match event {
            egui_node_graph::NodeResponse::CreatedNode(node_id) => {
                let node = &mut self.graph_state.graph[node_id];
                
                let new_shader = new_shader_data(display, egui_glium, node.user_data.template);
                node.user_data.result = Some(new_shader.clone_tex_id());
                self.shaders.insert(node_id, new_shader);

                if node.user_data.template == NodeTypes::Output {
                    self.output_nodes.push(node_id)
                }
            },

            NodeResponse::DeleteNodeFull { node_id, .. } => {
                if let Some(output_index) = self.output_nodes.iter().position(|id| *id == node_id){
                    self.output_nodes.swap_remove(output_index);
                }

                // slotmap may pre destroy this
                self.shaders.remove(node_id);
            }
            _ => {}
        }
    }

    fn render_node_and_inputs(&self, frame: &mut Frame, node_id: NodeId, rendered: &mut Vec<NodeId>) {
        if rendered.contains(&node_id){
            return;
        }

        let shader_data = &self.shaders[node_id];
        shader_data.render(frame);

        rendered.push(node_id);

        for input in &self.graph_state.graph[node_id].inputs {
            self.render_node_and_inputs(frame, self.graph_state.graph[input.1].node, rendered)
        }
    }

    fn render_shaders(&mut self, display: &Display){
        let mut rendered_nodes = vec![];

        for output_id in &self.output_nodes {
            // let node = self.state.graph[*output_id];
            let mut frame = display.draw();
            self.render_node_and_inputs(&mut frame, *output_id, &mut rendered_nodes);
            frame.finish().unwrap();
        }
    }

    pub fn draw(&mut self, display: &Display, egui_glium: &mut EguiGlium){
        let mut frame = display.draw();

        let mut graph_response = None;

        let _needs_repaint = egui_glium.run(display, |ctx| {
            graph_response = Some(self.draw_egui(ctx));
        });

        if let Some(response) = graph_response {
            for event in response.node_responses{
                self.node_event(display, egui_glium, event);
            }
        }

        self.render_shaders(display);
        
        frame.clear_color_and_depth((1.,1.,1.,1.), 0.);

        egui_glium.paint(display, &mut frame);

        frame.finish().unwrap();
    }

    fn draw_egui(&mut self, ctx: &egui::Context) -> egui_node_graph::GraphResponse<GraphResponse, NodeData> {
        egui::TopBottomPanel::top("top").show(ctx, |ui| {
            ui.heading("Hello World!");
            egui::menu::bar(ui, |ui| {
                egui::widgets::global_dark_light_mode_switch(ui);
            })
        });

        egui::CentralPanel::default().show(ctx, |ui| {
            let graph_resp = self.graph_state.draw_graph_editor(ui, AllNodeTypes);

            let output = self.output_nodes.first()
                .map(|output_node_id| self.shaders.get(*output_node_id))
                .flatten();

            if let Some(cache) = output {
                ui.image(cache.clone_tex_id(), [512., 512.]);
            }

            graph_resp
        }).inner
    }
}

// impl eframe::App for ShaderNodeGraph {
//     fn update(&mut self, ctx: &egui::Context, frame: &mut eframe::Frame) {
//         self.draw(ctx);
//     }
// }