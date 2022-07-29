use std::rc::Rc;

use egui::TextureId;
use egui_glium::EguiGlium;
use glium::{texture::SrgbTexture2d, framebuffer::{SimpleFrameBuffer, DefaultFramebuffer}, backend::Facade};
use strum::{EnumIter, IntoStaticStr};

pub struct NodeData {
    pub template: NodeTypes,
    pub result: Option<TextureId>
    // pub texture_cache: Option<ShaderData>
}

#[derive(PartialEq, Eq, IntoStaticStr, Clone, Copy)]
pub enum NodeConnectionTypes {
    // FrameBuffer,
    Texture2D
}

#[derive(Debug)]
pub enum NodeValueTypes {
    Vec2 { value: [f32; 2] },
    Float { value: f32 },
    None
}

#[derive(Clone, Copy, IntoStaticStr, EnumIter, PartialEq, Eq, PartialOrd, Ord)]
pub enum NodeTypes {
    Instances,
    Feedback,
    Sdf,
    Output
}

///Default connections
impl From<&NodeTypes> for NodeConnectionTypes {
    fn from(ty: &NodeTypes) -> Self {
        match ty {
            NodeTypes::Instances => Self::Texture2D,
            NodeTypes::Feedback => Self::Texture2D,
            NodeTypes::Sdf => Self::Texture2D,
            NodeTypes::Output => Self::Texture2D,
        }
    }
}

#[derive(Debug, Clone)]
pub enum GraphResponse {}


#[derive(Default)]
pub struct GraphState {}