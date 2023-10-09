use super::{animator::Animator, node_types::NodeType, GraphShaderProcessor, GraphUpdateListener};
use crate::{
    common::{connections::ConnectionType, def::UiValue},
    def::GetUiValue,
    GetTemplate,
};

use egui_node_graph::{NodeId, UserResponseTrait};
use glium::backend::Facade;
use serde::{Deserialize, Serialize};
use slotmap::{SecondaryMap, SparseSecondaryMap};
use std::{
    collections::{HashSet},
    fmt::{Debug, Display},
    time::{Instant},
};

#[derive(Clone, Debug)]
pub struct NodeError {
    pub text: String,
    pub when: Instant,
}

impl From<anyhow::Error> for NodeError {
    fn from(err: anyhow::Error) -> Self {
        Self {
            text: format!("{err:?}"),
            when: Instant::now(),
        }
    }
}

#[derive(Serialize, Deserialize, Clone)]
pub struct NodeData {
    pub template: NodeType,
}

impl GetTemplate for NodeData {
    fn template(&self) -> &NodeType {
        &self.template
    }
    fn template_mut(&mut self) -> &mut NodeType {
        &mut self.template
    }
}

impl NodeData {
    pub fn new(template: NodeType) -> Self {
        Self { template }
    }
}

impl Debug for NodeData {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut binding = f.debug_struct("UiNodeData");
        binding.field("template", &self.template).finish()
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UniqueNodeName {
    pub name: String,
    pub num: usize,
    pub code_name: String,
}

impl UniqueNodeName {
    pub fn new(name: String, num: usize) -> Self {
        let mut code_name = name.to_lowercase().replace(" ", "_");

        if 0 < num {
            code_name.push_str(&num.to_string());
        }

        Self {
            name,
            num,
            code_name,
        }
    }
}

impl Display for UniqueNodeName {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{code_name}", code_name = self.code_name)
    }
}

#[derive(Default, Serialize, Deserialize)]
#[serde(default)]
pub struct GraphState {
    pub node_names: SecondaryMap<NodeId, UniqueNodeName>,

    pub param_with_popup: Option<(NodeId, String)>,
    pub visible_nodes: HashSet<NodeId>,

    #[serde(skip)]
    pub processor: GraphShaderProcessor,

    pub animator: Animator,
}

use crate::graph::graph_change_listener::MultipleUpdatesListener;

impl GraphState {
    pub fn from_persistent_state<N: GetTemplate, V>(
        graph: &mut egui_node_graph::Graph<N, ConnectionType, V>,
        node_names: SecondaryMap<NodeId, UniqueNodeName>,
        animator: Animator,
        facade: &impl Facade,
    ) -> anyhow::Result<Self> {
        Ok(Self {
            node_names,
            animator,
            param_with_popup: None,
            visible_nodes: Default::default(),
            processor: GraphShaderProcessor::new_from_graph(graph, facade)?,
        })
    }
}

impl<N: GetTemplate, V> GraphUpdateListener<N, ConnectionType, V> for GraphState {
    fn graph_event(
        &mut self,
        graph: &mut egui_node_graph::Graph<N, ConnectionType, V>,
        facade: &impl glium::backend::Facade,
        event: super::GraphChangeEvent,
    ) -> anyhow::Result<()> {
        self.processor.graph_event(graph, facade, event)?;
        self.animator.graph_event(graph, facade, event)?;
        Ok(())
    }
}

impl GraphState {
    pub fn update<N: GetTemplate, C, V: GetUiValue>(
        &mut self,
        graph: &mut egui_node_graph::Graph<N, C, V>,
        facade: &impl glium::backend::Facade,
    ) -> SparseSecondaryMap<NodeId, anyhow::Error> {
        let errors = self.processor.update(graph, facade);
        self.animator.update(graph);
        errors
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CustomGraphResponse;
impl UserResponseTrait for CustomGraphResponse {}

pub type Node = egui_node_graph::Node<NodeData>;
pub type NodeResponse = egui_node_graph::NodeResponse<CustomGraphResponse, NodeData>;
pub type GraphResponse = egui_node_graph::GraphResponse<CustomGraphResponse, NodeData>;
pub type Graph = egui_node_graph::graph::Graph<NodeData, ConnectionType, UiValue>;
pub type GraphEditorState =
    egui_node_graph::GraphEditorState<NodeData, ConnectionType, UiValue, NodeType, GraphState>;
