
use std::{rc::{Weak}, cell::RefCell, path::PathBuf, time::Instant};

use egui::{Rgba};
use egui_node_graph::{GraphEditorState, UserResponseTrait, NodeResponse};
use glam::{Mat4, Vec3, EulerRot};
use glium::{uniforms::{AsUniformValue, UniformValue}};
use serde::{Serialize, Deserialize};
use strum::{Display};


use crate::{textures::UiTexture, common::def::{ConnectionType, UiValue}};

use super::node_types::NodeType;

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

#[derive(Serialize, Deserialize)]
pub struct GraphState;

pub type ShaderNodeResponse = NodeResponse<GraphResponse, NodeData>;
pub type EditorState = GraphEditorState<NodeData, ConnectionType, UiValue, NodeType, GraphState>;
pub type Graph = egui_node_graph::graph::Graph<NodeData, ConnectionType, UiValue>;