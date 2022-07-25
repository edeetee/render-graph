use egui_node_graph::GraphEditorState;

mod def;
mod logic;
mod helpers;

use def::*;

type EditorState = GraphEditorState<NodeData, NodeConnectionTypes, ValueTypes, NodeTypes, GraphState>;

pub struct NodeGraph{

}