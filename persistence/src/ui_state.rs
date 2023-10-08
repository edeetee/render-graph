use egui_node_graph::{AnyParameterId, NodeId};
use serde::{Deserialize, Serialize};

#[derive(Default, Serialize, Deserialize, Clone)]
pub struct GraphUiState {
    pub view_state: ViewState,
    pub node_selection_actor: Option<NodeSelectionActor>,
    pub last_connection_in_progress: Option<(NodeId, AnyParameterId)>,
}

#[derive(Default, Serialize, Deserialize, PartialEq, Clone)]
pub enum ViewState {
    #[default]
    Graph,
    Output,
}

impl ViewState {
    pub fn toggle(&mut self) {
        *self = match self {
            Self::Graph => Self::Output,
            Self::Output => Self::Graph,
        }
    }
}
#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum NodeSelectionActor {
    Mouse(egui::Pos2),
    DraggingOutput(egui::Pos2, NodeId, AnyParameterId),
}

impl NodeSelectionActor {
    pub fn pos(&self) -> egui::Pos2 {
        match self {
            Self::Mouse(pos) => *pos,
            Self::DraggingOutput(pos, _, _) => *pos,
        }
    }

    pub fn connection(&self) -> Option<(NodeId, AnyParameterId)> {
        match self {
            Self::Mouse(_) => None,
            Self::DraggingOutput(_, node_id, param_id) => Some((*node_id, *param_id)),
        }
    }
}
