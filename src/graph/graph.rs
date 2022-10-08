use std::{ops::{Index, IndexMut}};


use egui_node_graph::{GraphEditorState, NodeId, Node, InputParam, Graph, NodeTemplateTrait};

use slotmap::SecondaryMap;

use crate::{isf::meta::{IsfInfo}};

use super::{def::{GraphState, NodeData, GraphResponse, ConnectionType, UiValue, EditorState}, node_types::{AllNodeTypes, NodeTypes}, node_tree_ui::TreeState};

// #[derive(Default)]
pub struct ShaderGraph(pub(super) EditorState, TreeState);

impl Default for ShaderGraph {
    fn default() -> Self {
        Self(
            GraphEditorState::new(1.0, GraphState::default()),
            TreeState::default()
        )
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
    pub fn graph_ref(&self) -> &Graph<NodeData, ConnectionType, UiValue> {
        &self.0.graph
    }

    ///Call f for each node in correct order, ending on node_id\
    /// 
    /// # Type arguments
    /// OUT: type that may come out of a 
    pub fn map_with_inputs<FOnNode, OUT: Clone>(&self, node_id: NodeId, f_on_node: &mut FOnNode, cache: &mut SecondaryMap<NodeId, OUT>) -> OUT 
        where FOnNode: FnMut(NodeId, Vec<(&str, &InputParam<ConnectionType, UiValue>, Option<OUT>)>) -> OUT
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

    pub fn add_node(&mut self, node_kind: NodeTypes, position: egui::Pos2) -> NodeId {
        let new_node = self.0.graph.add_node(
            node_kind.node_graph_label(),
            node_kind.user_data(),
            |graph, node_id| node_kind.build_node(graph, node_id),
        );
        self.0.node_positions.insert(
            new_node,
            position,
        );
        self.0.node_order.push(new_node);

        new_node
    }

    pub fn draw(&mut self, ctx: &egui::Context) -> egui_node_graph::GraphResponse<GraphResponse, NodeData> {

        let mut new_node_ty = None;

        egui::SidePanel::left("tree_view").show(ctx, |ui| {

            if let Some(selected_item) = self.1.draw(ui) {

                match IsfInfo::new_from_path(&selected_item.path){
                    Ok(info) => {
                        new_node_ty = Some(NodeTypes::Isf { info });
                        
                    },
                    Err(e) => {
                        println!("{e}");
                    }
                }
            }

        });

        egui::CentralPanel::default().show(ctx, |ui| {
            egui::widgets::global_dark_light_mode_switch(ui);
            let mut responses = vec![];

            let editor_rect = ui.max_rect();

            if let Some(node_ty) = new_node_ty {
                let pos = editor_rect.left_top() - self.0.pan_zoom.pan;
                let new_node_id = self.add_node(node_ty, pos);
                responses.push(egui_node_graph::NodeResponse::CreatedNode(new_node_id));
            }

            if ui.ui_contains_pointer() {
                self.0.pan_zoom.pan += ctx.input().scroll_delta;
                // self.0.pan_zoom.zoom *= ctx.input().zoom_delta();
                // dbg!(self.0.pan_zoom.zoom);
            }

            let mut graph_resp = self.0.draw_graph_editor(ui, AllNodeTypes);

            graph_resp.node_responses.append(&mut responses);

            graph_resp
        }).inner
    }
}