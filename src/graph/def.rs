
use std::{rc::{Weak}, cell::RefCell, time::Instant, collections::{HashMap, HashSet}};
use egui_node_graph::{GraphEditorState, UserResponseTrait, NodeResponse, NodeId};
use serde::{Serialize, Deserialize};
use crate::{textures::UiTexture, common::{def::{UiValue}, animation::DataUpdater, connections::ConnectionType}};
use super::{node_types::NodeType};

#[derive(Clone)]
pub struct NodeError {
    pub text: String,
    pub when: Instant
}

impl From<anyhow::Error> for NodeError {
    fn from(err: anyhow::Error) -> Self {
        Self { text: format!("{err:?}"), when: Instant::now() }
    }
}

#[derive(Serialize, Deserialize)]
pub struct NodeData {
    pub template: NodeType,
    
    #[serde(skip)]
    pub texture: Weak<RefCell<UiTexture>>, // pub texture_cache: Option<ShaderData>
    #[serde(skip)]
    pub create_error: Option<NodeError>,
    #[serde(skip)]
    pub update_error: Option<NodeError>,
    #[serde(skip)]
    pub render_error: Option<NodeError>,
}

impl NodeData {
    pub fn new(template: NodeType) -> Self {
        Self {
            template,
            texture: Default::default(),
            create_error: Default::default(),
            update_error: Default::default(),
            render_error: Default::default(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GraphResponse;
impl UserResponseTrait for GraphResponse {}

#[derive(Default, Serialize, Deserialize)]
pub struct GraphState {
    pub animations: HashMap<(NodeId, String), DataUpdater>,
    pub param_with_popup: Option<(NodeId, String)>,
    pub visible_nodes: HashSet<NodeId>
}

pub type ShaderNodeResponse = NodeResponse<GraphResponse, NodeData>;
pub type EditorState = GraphEditorState<NodeData, ConnectionType, UiValue, NodeType, GraphState>;
pub type Graph = egui_node_graph::graph::Graph<NodeData, ConnectionType, UiValue>;