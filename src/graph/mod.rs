use egui_node_graph::GraphEditorState;

mod def;
mod logic;
mod helpers;
mod ui;

use def::*;

type EditorState = GraphEditorState<NodeData, NodeConnectionTypes, ValueTypes, NodeTypes, GraphState>;

pub struct NodeGraph{
    state: EditorState
}

impl NodeGraph {
    fn new() -> Self {
        Self { 
            state: GraphEditorState::new(1.0, GraphState {})
        }
    }
}