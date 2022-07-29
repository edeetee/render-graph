use eframe::glow::Framebuffer;
use egui::TextureId;
use egui_node_graph::{NodeId, NodeResponse};
use glium::texture::SrgbTexture2d;
use slotmap::SecondaryMap;
use super::def::{self, NodeData};



// pub struct ShaderNodeManager{
//     output_nodes: SecondaryMap<NodeId, Option<ShaderData>>
// }

// impl ShaderNodeManager {
//     pub fn new() -> Self {
//         Self { output_nodes: SecondaryMap::new() }
//     }

//     pub fn render(&self, frame: Frame){

//     }