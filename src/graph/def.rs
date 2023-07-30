
use std::{rc::{Weak}, cell::RefCell, time::Instant, collections::{HashMap, HashSet}, fmt::Debug};
use egui_node_graph::{UserResponseTrait, NodeId};
use serde::{Serialize, Deserialize};
use crate::{common::{def::{UiValue}, animation::DataUpdater, connections::ConnectionType}};
use super::{node_types::NodeType};

#[derive(Clone,Debug)]
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
pub struct UiNodeData {
    pub template: NodeType,

    // pub name: String,

    #[serde(skip)]
    #[cfg(feature="editor")]
    pub texture: Weak<RefCell<crate::textures::ui::UiTexture>>, // pub texture_cache: Option<ShaderData>

    #[serde(skip)]
    pub create_error: Option<NodeError>,
    #[serde(skip)]
    pub update_error: Option<NodeError>,
    #[serde(skip)]
    pub render_error: Option<NodeError>,
}

impl UiNodeData {
    pub fn new(template: NodeType) -> Self {
        Self {
            template,
            #[cfg(feature="editor")]
            texture: Default::default(),
            create_error: Default::default(),
            update_error: Default::default(),
            render_error: Default::default(),
        }
    }
}

impl Debug for UiNodeData {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("UiNodeData")
        .field("template", &self.template)
        .field("texture", &self.texture)
        .field("create_error", &self.create_error)
        .field("update_error", &self.update_error)
        .field("render_error", &self.render_error)
        .finish()
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GraphResponse;
impl UserResponseTrait for GraphResponse {}

#[derive(Default, Serialize, Deserialize)]
pub struct GraphState {
    #[serde(with = "vectorize")] 
    pub animations: HashMap<(NodeId, String), DataUpdater>,
    
    // #[serde(with = "vectorize")]
    // pub node_names: HashMap<NodeId, String>,

    pub param_with_popup: Option<(NodeId, String)>,
    pub visible_nodes: HashSet<NodeId>
}

pub type Node = egui_node_graph::Node<UiNodeData>;
pub type NodeResponse = egui_node_graph::NodeResponse<GraphResponse, UiNodeData>;
pub type GraphEditorState = egui_node_graph::GraphEditorState<UiNodeData, ConnectionType, UiValue, NodeType, GraphState>;
pub type Graph = egui_node_graph::graph::Graph<UiNodeData, ConnectionType, UiValue>;