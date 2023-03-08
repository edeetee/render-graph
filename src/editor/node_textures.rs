use std::{rc::Rc, cell::RefCell, borrow::BorrowMut};

use egui::TextureId;
use egui_glium::EguiGlium;
use egui_node_graph::{NodeId};
use glium::{backend::Facade, Surface, texture::SrgbTexture2d, framebuffer::SimpleFrameBuffer};
use ouroboros::self_referencing;
use slotmap::SecondaryMap;

use crate::textures::ui::UiTexture;


#[derive(Default)]
pub struct NodeTextures {
    inner: SecondaryMap<NodeId, Rc<RefCell<UiTexture>>>,
}

impl NodeTextures {
    pub fn new_from_graph(graph: &mut crate::graph::def::Graph, facade: &impl Facade, egui_glium: &mut EguiGlium) -> Self {
        let mut me = Self::default();

        for node in graph.nodes.values_mut() {
            me.add(facade, egui_glium, node);
        }

        me
    }

    pub fn add(&mut self, facade: &impl Facade, egui_glium: &mut EguiGlium, node: &mut crate::graph::def::Node) {
        let ui_texture = UiTexture::new(facade, egui_glium, (256, 256));
        let textures = Rc::new(RefCell::new(ui_texture));
        
        node.user_data.texture = Rc::downgrade(&textures);
        self.inner.insert(node.id, textures);
    }

    pub fn remove(&mut self, node_id: NodeId) {
        self.inner.remove(node_id);
    }

    pub fn copy_surface(&mut self, facade: &impl Facade, egui_glium: &mut EguiGlium, node_id: NodeId, surface: &impl Surface) {
        let ui_texture = &mut *(*self.inner[node_id]).borrow_mut();
        // let ui_texture = *self.inner
        
        ui_texture.update_size(facade, egui_glium, surface.get_dimensions());
        ui_texture.copy_from(surface);
    }
}