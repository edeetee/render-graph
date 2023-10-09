use super::ui_texture::UiTexture;
use delegate::delegate;
use egui_node_graph::{NodeId, NodeTemplateIter, NodeTemplateTrait};
use glium::uniforms::UniformValue;
use graph::{
    def::{AsUniformOptional, GetUiValue},
    GetTemplate, NodeError,
};
use serde::{Deserialize, Serialize};
use std::{cell::RefCell, rc::Weak, time::Duration};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct UiNodeData {
    #[serde(flatten)]
    pub inner: graph::UiNodeData,

    #[serde(skip)]
    pub create_error: Option<NodeError>,
    #[serde(skip)]
    pub update_error: Option<NodeError>,
    #[serde(skip)]
    pub render_error: Option<NodeError>,
    #[serde(skip)]
    pub render_time: Option<Duration>,

    #[serde(skip)]
    pub texture: Weak<RefCell<UiTexture>>, // pub texture_cache: Option<ShaderData>
}

impl UiNodeData {
    pub fn update_time_smoothed(&mut self, new_time: Duration) {
        let old_time = self.render_time.unwrap_or(new_time);

        self.render_time = Some(old_time.mul_f32(0.9) + new_time.mul_f32(0.1));
    }
}

impl From<graph::UiNodeData> for UiNodeData {
    fn from(inner: graph::UiNodeData) -> Self {
        Self {
            inner,
            create_error: Default::default(),
            update_error: Default::default(),
            render_error: Default::default(),
            render_time: Default::default(),
            texture: Weak::new(),
        }
    }
}

impl GetTemplate for UiNodeData {
    delegate! {
        to self.inner {
            fn template(&self) -> &graph::NodeType;
            fn template_mut(&mut self) -> &mut graph::NodeType;
        }
    }
}

#[derive(Default, Clone, Debug, Serialize, Deserialize)]
#[repr(transparent)]
pub struct UiValue(pub graph::def::UiValue);

impl From<graph::def::UiValue> for UiValue {
    fn from(inner: graph::def::UiValue) -> Self {
        Self(inner)
    }
}

impl GetUiValue for UiValue {
    delegate! {
        to self.0 {
            fn ui_value(&self) -> &graph::def::UiValue;
            fn ui_value_mut(&mut self) -> &mut graph::def::UiValue;
        }
    }
}

impl AsUniformOptional for UiValue {
    delegate! {
        to self.0 {
            fn as_uniform_optional(&self) -> Option<UniformValue>;
        }
    }
}

#[derive(Clone, Debug)]
pub struct CustomGraphResponse;
impl egui_node_graph::UserResponseTrait for CustomGraphResponse {}

pub type GraphResponse = egui_node_graph::GraphResponse<CustomGraphResponse, UiNodeData>;
pub type NodeResponse = egui_node_graph::NodeResponse<CustomGraphResponse, UiNodeData>;
pub type ConnectionType = graph::connections::ConnectionType;
pub type GraphState = graph::GraphState;
pub type Node = egui_node_graph::Node<UiNodeData>;
pub type Graph = egui_node_graph::Graph<UiNodeData, ConnectionType, UiValue>;

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct NodeType(pub graph::NodeType);

impl NodeTemplateTrait for NodeType {
    type NodeData = UiNodeData;
    type DataType = ConnectionType;
    type ValueType = UiValue;
    type UserState = GraphState;

    fn node_finder_label(&self, _user_state: &mut Self::UserState) -> std::borrow::Cow<str> {
        self.0.get_name().into()
    }

    fn node_graph_label(&self, user_state: &mut Self::UserState) -> String {
        self.node_finder_label(user_state).into()
    }

    fn user_data(&self, _user_state: &mut Self::UserState) -> Self::NodeData {
        graph::UiNodeData::new(self.0.clone()).into()
    }

    fn build_node(
        &self,
        graph: &mut egui_node_graph::Graph<Self::NodeData, Self::DataType, Self::ValueType>,
        _user_state: &mut Self::UserState,
        node_id: NodeId,
    ) {
        for input in self.0.get_input_types() {
            if let Some(kind) = input.kind() {
                graph.add_input_param(
                    node_id,
                    input.name,
                    input.ty,
                    input.value.into(),
                    kind,
                    true,
                );
            }
        }

        for output in self.0.get_output_types() {
            graph.add_output_param(node_id, output.name, output.ty);
        }
    }
}

pub struct AllNodeTypes;

impl NodeTemplateIter for AllNodeTypes {
    type Item = NodeType;

    fn all_kinds(&self) -> Vec<Self::Item> {
        graph::NodeType::defaults()
            .into_iter()
            .map(NodeType)
            .collect()
    }
}

pub type GraphEditorState =
    egui_node_graph::GraphEditorState<UiNodeData, ConnectionType, UiValue, NodeType, GraphState>;
