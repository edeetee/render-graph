use std::ops::{Index, IndexMut};

use eframe::glow::Shader;
use egui_node_graph::{GraphEditorState, NodeId, Node, InputParam, OutputParam, InputId, OutputId, Graph};
use glium::framebuffer::SimpleFrameBuffer;
use slotmap::SecondaryMap;

use super::{shader_graph_processor::EditorState, def::{GraphState, NodeData, GraphResponse, NodeConnectionTypes, NodeValueTypes}, node_shader::NodeShader, logic::AllNodeTypes};

// #[derive(Default)]
pub struct ShaderGraph(pub(super) EditorState);

impl Default for ShaderGraph {
    fn default() -> Self {
        Self(GraphEditorState::new(1.0, GraphState::default()))
    }
}

impl Index<NodeId> for ShaderGraph {
    type Output = Node<NodeData>;

    fn index(&self, index: NodeId) -> &Self::Output {
        &self.0.graph[index]
    }
}

impl IndexMut<NodeId> for ShaderGraph {
    fn index_mut(&mut self, index: NodeId) -> &mut Self::Output {
        &mut self.0.graph[index]
    }
}

impl ShaderGraph {
    // pub fn inputs(&self, node_id: NodeId) -> impl Iterator<Item = &InputParam<NodeConnectionTypes, NodeValueTypes>>{
    //     self.0.graph[node_id].inputs(&self.0.graph)
    //     // if self.graph.0.graph.connections[self.graph.0.graph[node_id].input_ids().next().unwrap()]
    // }

    // pub fn outputs(&self, node_id: NodeId) -> impl Iterator<Item = &OutputParam<NodeConnectionTypes>>{
    //     self.0.graph[node_id].outputs(&self.0.graph)
    //     // if self.graph.0.graph.connections[self.graph.0.graph[node_id].input_ids().next().unwrap()]
    // }
    pub fn graph_ref(&self) -> &Graph<NodeData, NodeConnectionTypes, NodeValueTypes> {
        &self.0.graph
    }

    pub fn connection(&self, input: InputId) -> Option<OutputId> {
        self.0.graph.connection(input)
    }

    ///Call f for each node in correct order, ending on node_id
    pub fn map_to<T>(&self, node_id: NodeId, f: &mut impl FnMut(NodeId, Vec<(NodeId, T)>) -> T) -> T{
        let mut prev_vals = vec![];

        //call preceeding nodes first
        for (_, input_id) in &self.0.graph[node_id].inputs {
            if let Some(output_id) = self.0.graph.connection(*input_id){

                let computing_node_id = self.0.graph[output_id].node;
                let computation_result = self.map_to(computing_node_id, f);

                prev_vals.push((computing_node_id, computation_result));
            }
        }

        f(node_id, prev_vals)
    }

    pub fn draw(&mut self, ctx: &egui::Context) -> egui_node_graph::GraphResponse<GraphResponse, NodeData> {
        egui::TopBottomPanel::top("top").show(ctx, |ui| {
            ui.heading("Hello World!");
            egui::menu::bar(ui, |ui| {
                egui::widgets::global_dark_light_mode_switch(ui);
            })
        });

        egui::CentralPanel::default().show(ctx, |ui| {
            let graph_resp = self.0.draw_graph_editor(ui, AllNodeTypes);

            // let output = self.output_targets.iter().next()
            //     .map(|(output_node_id, _)| self.shaders.get(output_node_id))
            //     .flatten();

            // if let Some(cache) = output {
            //     ui.image(cache.clone_tex_id(), [512., 512.]);
            // }

            graph_resp
        }).inner
    }
}

