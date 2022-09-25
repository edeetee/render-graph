use std::ops::{Index, IndexMut};

use egui_node_graph::{GraphEditorState, NodeId, Node, InputParam, Graph};

use super::{def::{GraphState, NodeData, GraphResponse, NodeConnectionTypes, NodeValueTypes, EditorState}, node_types::AllNodeTypes};

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

pub type InputData<'a> = (&'a String, &'a InputParam<NodeConnectionTypes, NodeValueTypes>, Option<NodeId>);

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

    // pub fn connection(&self, input: InputId) -> Option<OutputId> {
    //     self.0.graph.connection(input)
    // }

    // pub type ComputedInput<T> = (&String, &NodeId, Option<T>);

    ///Call f for each node in correct order, ending on node_id
    pub fn map_with_inputs(&self, node_id: NodeId, f: &mut impl FnMut(NodeId, Vec<InputData<'_>>)) {
        // let inputs = self.0.graph[node_id].inputs;

        let computed_inputs = self.0.graph[node_id].inputs.iter()
            .map(|(name, input_id)| {
                //if input is connected, generate the value

                (name, &self.0.graph[*input_id], self.0.graph.connection(*input_id).map(|output_id| {
                    //we get to process a node!
                    let computing_node_id = self.0.graph[output_id].node;

                    //process the node first
                    self.map_with_inputs(computing_node_id, f);

                    computing_node_id
                }))
            }).collect();

        f(node_id, computed_inputs)
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

