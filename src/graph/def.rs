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
pub enum ValueTypes {
    Vec2 { value: [f32; 2] },
    Float { value: f32 },
    None
}

impl From<&NodeConnectionTypes> for ValueTypes {
    fn from(connection: &NodeConnectionTypes) -> Self {
        match connection {
            NodeConnectionTypes::FrameBuffer => ValueTypes::None,
            NodeConnectionTypes::Texture2D => ValueTypes::None,
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

pub struct GraphState {

}