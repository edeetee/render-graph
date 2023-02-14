use std::{ops::{Index, IndexMut}};

use egui::{Rect, Pos2, Vec2};
use egui_node_graph::{GraphEditorState, NodeId, Node, InputParam, Graph, NodeTemplateTrait};

use serde::Serialize;
use slotmap::{SecondaryMap};
use crate::common::def::{ConnectionType, UiValue};

use super::{def::{GraphState, NodeData, GraphResponse, EditorState}, node_types::{AllNodeTypes, NodeType}, node_tree_ui::TreeState};

// #[derive(Default)]
pub struct ShaderGraph { 
    pub editor: EditorState, 
    pub state: GraphState,
    pub tree: TreeState 
}


impl Default for ShaderGraph {
    fn default() -> Self {
        Self { 
            editor: GraphEditorState::new(1.0), 
            state: GraphState::default(),
            tree: TreeState::default()
        }
    }
}

impl Index<NodeId> for ShaderGraph {
    type Output = Node<NodeData>;

    fn index(&self, index: NodeId) -> &Self::Output {
        &self.editor.graph[index]
    }
}

impl IndexMut<NodeId> for ShaderGraph {
    fn index_mut(&mut self, index: NodeId) -> &mut Self::Output {
        &mut self.editor.graph[index]
    }
}

pub type InputParams<'a> = Vec<(&'a str, &'a InputParam<ConnectionType, UiValue>)>;
pub type ProcessedInputs<'a, OUT> = Vec<(&'a str, &'a InputParam<ConnectionType, UiValue>, Option<OUT>)>;

impl ShaderGraph {
    pub fn graph_ref(&self) -> &Graph<NodeData, ConnectionType, UiValue> {
        &self.editor.graph
    }

    pub fn input_params(&self, node_id: NodeId) -> InputParams<'_> {
        self.graph_ref()[node_id].inputs.iter().map(|(name, input_id)| {
            (name.as_str(), &self.graph_ref()[*input_id])
        }).collect()
    }

    ///Call f for each node in correct order, ending on node_id\
    /// 
    /// # Type arguments
    /// OUT: type that may come out of a 
    pub fn map_with_inputs<FOnNode, OUT: Clone>(&self, node_id: NodeId, f_on_node: &mut FOnNode, cache: &mut SecondaryMap<NodeId, Option<OUT>>) -> Option<OUT> 
        where FOnNode: FnMut(NodeId, ProcessedInputs<'_, OUT>) -> Option<OUT>
    {
        let computed_inputs = self.editor.graph[node_id].inputs.iter()
            .map(|(name, input_id)| {
                //if input is connected, generate the value

                let process_input = self.editor.graph.connection(*input_id).map(|output_id| {
                    //we get to process a node!
                    let input_node_id = self.editor.graph[output_id].node;

                    //add input to cache if doesn't exist
                    if !cache.contains_key(input_node_id){
                        let value = self.map_with_inputs(input_node_id, f_on_node, cache);
                        cache.insert(input_node_id, value);
                    }

                    cache[input_node_id].clone()
                }).flatten();

                let input_param = &self.editor.graph[*input_id];

                (name.as_str(), input_param, process_input)
            })
            .collect();

        let result = f_on_node(node_id, computed_inputs);

        result
    }

    pub fn add_node(&mut self, node_kind: &NodeType, position: egui::Pos2) -> NodeId {
        // println!("Adding node {node_kind:#?}");

        let new_node = self.editor.graph.add_node(
            node_kind.node_graph_label(&mut self.state),
            node_kind.user_data(&mut self.state),
            |graph, node_id| node_kind.build_node(graph, &mut self.state, node_id),
        );
        self.editor.node_positions.insert(
            new_node,
            position,
        );
        self.editor.node_order.push(new_node);

        new_node
    }

    pub fn draw(&mut self, ctx: &egui::Context) -> egui_node_graph::GraphResponse<GraphResponse, NodeData> {
        let mut new_node_ty = None;

        egui::SidePanel::left("tree_view").show(ctx, |ui| {
            if let Some(selected_item) = self.tree.draw(ui) {
                new_node_ty = Some(selected_item.ty.clone());
            }
        });
        

        egui::CentralPanel::default().show(ctx, |ui| {
            ui.set_clip_rect(ctx.available_rect());
            egui::widgets::global_dark_light_mode_switch(ui);

            let mut responses = vec![];

            let editor_rect = ui.max_rect();

            if let Some(node_ty) = new_node_ty {
                let pos = editor_rect.left_top() - self.editor.pan_zoom.pan;
                let new_node_id = self.add_node(&node_ty, pos);
                responses.push(egui_node_graph::NodeResponse::CreatedNode(new_node_id));
            }

            if ui.ui_contains_pointer() {
                self.editor.pan_zoom.pan += ctx.input().scroll_delta;

                if let Some(point) = ctx.input().pointer.hover_pos() {
                    let zoom_delta = ctx.input().zoom_delta();
                    self.editor.pan_zoom.adjust_zoom(zoom_delta, point.to_vec2(), 0.001, 100.0);
                }
                // self.0.pan_zoom.zoom *= ctx.input().zoom_delta();
                // dbg!(self.0.pan_zoom.zoom);
            }

            let mut graph_resp = self.editor.draw_graph_editor(ui, AllNodeTypes, &mut self.state);
            self.editor.node_finder = None;
            graph_resp.node_responses.append(&mut responses);

            graph_resp
        }).inner
    }
}