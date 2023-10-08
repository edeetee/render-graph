use super::{
    animator::Animator, node_types::NodeType, GraphShaderProcessor, GraphUpdateListener,
    GraphUpdater,
};
use crate::{
    common::{animation::DataUpdater, connections::ConnectionType, def::UiValue},
    def::GetUiValue,
    GetTemplate,
};

use egui_node_graph::{NodeId, UserResponseTrait};
use glium::backend::Facade;
use serde::{Deserialize, Serialize};
use slotmap::SecondaryMap;
use std::{
    cell::RefCell,
    collections::{HashMap, HashSet},
    fmt::{Debug, Display},
    rc::Weak,
    time::{Duration, Instant},
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
pub struct UiNodeData {
    pub template: NodeType,

    #[serde(skip)]
    pub create_error: Option<NodeError>,
    #[serde(skip)]
    pub update_error: Option<NodeError>,
    #[serde(skip)]
    pub render_error: Option<NodeError>,
    #[serde(skip)]
    pub render_time: Option<Duration>,
}

impl GetTemplate for UiNodeData {
    fn template(&self) -> &NodeType {
        &self.template
    }
    fn template_mut(&mut self) -> &mut NodeType {
        &mut self.template
    }
}

impl UiNodeData {
    pub fn new(template: NodeType) -> Self {
        Self {
            template,
            create_error: Default::default(),
            update_error: Default::default(),
            render_error: Default::default(),
            render_time: Default::default(),
        }
    }

    pub fn update_time_smoothed(&mut self, new_time: Duration) {
        let old_time = self.render_time.unwrap_or(new_time);

        self.render_time = Some(old_time.mul_f32(0.9) + new_time.mul_f32(0.1));
    }
}

impl Debug for UiNodeData {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut binding = f.debug_struct("UiNodeData");
        let d = binding.field("template", &self.template);

        #[cfg(feature = "editor")]
        d.field("texture", &self.texture);

        d.field("create_error", &self.create_error)
            .field("update_error", &self.update_error)
            .field("render_error", &self.render_error)
            .finish()
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

impl<N: GetTemplate, C, V: GetUiValue> GraphUpdater<N, C, V> for GraphState {
    fn update(
        &mut self,
        graph: &mut egui_node_graph::Graph<N, C, V>,
        facade: &impl glium::backend::Facade,
    ) -> anyhow::Result<()> {
        self.processor.update(graph, facade)?;
        self.animator.update(graph, facade)?;
        Ok(())
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CustomGraphResponse;
impl UserResponseTrait for CustomGraphResponse {}

pub type Node = egui_node_graph::Node<UiNodeData>;
pub type NodeResponse = egui_node_graph::NodeResponse<CustomGraphResponse, UiNodeData>;
pub type GraphResponse = egui_node_graph::GraphResponse<CustomGraphResponse, UiNodeData>;
pub type Graph = egui_node_graph::graph::Graph<UiNodeData, ConnectionType, UiValue>;
pub type GraphEditorState =
    egui_node_graph::GraphEditorState<UiNodeData, ConnectionType, UiValue, NodeType, GraphState>;
