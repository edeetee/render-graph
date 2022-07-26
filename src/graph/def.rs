use egui_node_graph::{GraphEditorState, NodeId};
use strum::{EnumIter, IntoStaticStr, AsRefStr};


pub struct NodeData {
    pub template: NodeTypes
}

#[derive(PartialEq, Eq, IntoStaticStr, Clone, Copy)]
pub enum NodeConnectionTypes {
    FrameBuffer,
    Texture2D
}

#[derive(Copy, Clone, Debug)]
pub enum NodeValueTypes {
    Vec2 { value: [f32; 2] },
    Float { value: f32 },
    None
}

impl From<&NodeConnectionTypes> for NodeValueTypes {
    fn from(connection: &NodeConnectionTypes) -> Self {
        match connection {
            NodeConnectionTypes::FrameBuffer => NodeValueTypes::None,
            NodeConnectionTypes::Texture2D => NodeValueTypes::None,
        }
    }
}

#[derive(Clone, Copy, IntoStaticStr, EnumIter)]
pub enum NodeTypes {
    Instances,
    Feedback,
    Sdf,
    Output
}

#[derive(Debug, Clone)]
pub enum GraphResponse {
    None
}

pub struct GraphState {
    // outputs: Vec<NodeId>
}