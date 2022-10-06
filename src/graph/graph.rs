use std::ops::{Index, IndexMut};

use egui::Ui;
use egui_node_graph::{GraphEditorState, NodeId, Node, InputParam, Graph, NodeTemplateTrait};
use slotmap::SecondaryMap;

use super::{def::{GraphState, NodeData, GraphResponse, NodeConnectionTypes, NodeValueTypes, EditorState}, node_types::{AllNodeTypes, NodeTypes}};

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
    pub fn graph_ref(&self) -> &Graph<NodeData, NodeConnectionTypes, NodeValueTypes> {
        &self.0.graph
    }

    ///Call f for each node in correct order, ending on node_id\
    /// 
    /// # Type arguments
    /// OUT: type that may come out of a 
    pub fn map_with_inputs<FOnNode, OUT: Clone>(&self, node_id: NodeId, f_on_node: &mut FOnNode, cache: &mut SecondaryMap<NodeId, OUT>) -> OUT 
        where FOnNode: FnMut(NodeId, Vec<(&str, &InputParam<NodeConnectionTypes, NodeValueTypes>, Option<OUT>)>) -> OUT
    {
        let computed_inputs = self.0.graph[node_id].inputs.iter()
            .map(|(name, input_id)| {
                //if input is connected, generate the value

                let process_input = self.0.graph.connection(*input_id).map(|output_id| {
                    //we get to process a node!
                    let input_node_id = self.0.graph[output_id].node;

                    //add input to cache if doesn't exist
                    if !cache.contains_key(input_node_id){
                        let value = self.map_with_inputs(input_node_id, f_on_node, cache);
                        cache.insert(input_node_id, value);
                    }

                    cache[input_node_id].clone()
                });

                let input_param = self.0.graph.get_input(*input_id);

                (name.as_str(), input_param, process_input)
            })
            .collect();

        let result = f_on_node(node_id, computed_inputs);

        result
    }

    pub fn draw(&mut self, ctx: &egui::Context) -> egui_node_graph::GraphResponse<GraphResponse, NodeData> {

        egui::CentralPanel::default().show(ctx, |ui| {
            egui::widgets::global_dark_light_mode_switch(ui);

            let graph_resp = self.0.draw_graph_editor(ui, AllNodeTypes);


            graph_resp
        }).inner
    }
}

struct CustomNodeSelector {}

impl CustomNodeSelector {
    fn draw(&self, graph: &mut EditorState, ui: &mut Ui) -> Option<NodeTypes> {
        let all_kinds = NodeTypes::get_all();

        for kind in all_kinds {
            let kind_name = kind.node_finder_label().to_string();
            if ui.selectable_label(false, kind_name).clicked() {
                return Some(kind);
            }

        }

        None
    }
}