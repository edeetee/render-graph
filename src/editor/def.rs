use std::collections::{HashSet, HashMap};

use egui_node_graph::{NodeId, UserResponseTrait};
use serde::{Serialize, Deserialize};

use crate::{common::{animation::DataUpdater, connections::ConnectionType, def::UiValue}, graph::{def::{GraphState, UiNodeData}, node_types::NodeType}};

#[derive(Default, Serialize, Deserialize)]
pub struct GraphUiState {
    pub inner: GraphState,
    pub param_with_popup: Option<(NodeId, String)>,
    pub visible_nodes: HashSet<NodeId>
}

impl From<GraphState> for GraphUiState {
    fn from(inner: GraphState) -> Self {
        Self {
            inner,
            ..Default::default()
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GraphResponse;
impl UserResponseTrait for GraphResponse {}

pub type GraphEditorState = egui_node_graph::GraphEditorState<UiNodeData, ConnectionType, UiValue, NodeType, GraphState>;