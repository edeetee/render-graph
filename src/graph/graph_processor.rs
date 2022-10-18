use std::{
    rc::Rc, time::SystemTime, cell::RefCell,
};

use egui_glium::EguiGlium;
use egui_node_graph::{NodeId, NodeResponse};
use glium::{
    backend::Facade,
    Display,
    framebuffer::{RenderBuffer, SimpleFrameBuffer}, Surface, Texture2d, uniforms::AsUniformValue,
};

use ouroboros::self_referencing;
use slotmap::{SecondaryMap, SparseSecondaryMap};
use crate::textures::{UiTexture, TextureManager};

use super::{
    def::{self, *},
    graph::ShaderGraph,
    node_types::NodeType,
    node_shader::ShaderInputs, node_shader::NodeShader, node_update::{NodeUpdate},
};

extern crate gl;

#[self_referencing]
pub struct OutputTarget {
    rb: RenderBuffer,

    #[borrows(rb)]
    #[covariant]
    fb: SimpleFrameBuffer<'this>,
}

#[derive(Default)]
pub struct ShaderGraphProcessor {
    graph: ShaderGraph,
    texture_manager: TextureManager,

    output_targets: SparseSecondaryMap<NodeId, OutputTarget>,
    node_textures: SecondaryMap<NodeId, Rc<RefCell<UiTexture>>>,
    shaders: SecondaryMap<NodeId, NodeShader>,
    versions: SecondaryMap<NodeId, SystemTime>,

    updaters: SecondaryMap<NodeId, NodeUpdate>
}

impl ShaderGraphProcessor {
    pub fn new() -> Self {
        Default::default()
    }

    fn add_dangling_output(&mut self, facade: &impl Facade, node_id: NodeId) {
        // let is_output_target = node.outputs(&self.graph.graph_ref()).any(|o| o.typ == NodeConnectionTypes::Texture2D);

        let output_target = OutputTargetBuilder {
            rb: RenderBuffer::new(
                facade,
                glium::texture::UncompressedFloatFormat::F32F32F32F32,
                512,
                512,
            )
            .unwrap(),
            fb_builder: |rb| SimpleFrameBuffer::new(facade, rb).unwrap(),
        }
        .build();

        self.output_targets.insert(node_id, output_target);
    }

    pub fn node_event(
        &mut self,
        facade: &impl Facade,
        egui_glium: &mut EguiGlium,
        event: NodeResponse<def::GraphResponse, NodeData>,
    ) {
        match event {
            egui_node_graph::NodeResponse::CreatedNode(node_id) => {
                // self.node_textures.insert(node_id, textures);
                let textures = Rc::new(RefCell::new(UiTexture::new(facade, egui_glium, (256, 256))));
                self.graph[node_id].user_data.texture = Rc::downgrade(&textures);
                self.node_textures.insert(node_id, textures);

                let node = &self.graph[node_id];

                let template = &node.user_data.template;

                match template {
                    NodeType::Isf { info } => {
                        self.versions
                            .insert(node_id, info.path.metadata().unwrap().modified().unwrap());
                    }
                    _ => {}
                }

                //only add if needed ()s
                if let Some(shader) = NodeShader::new(template, facade) {
                    match shader {
                        Ok(shader) => {
                            self.shaders.insert(node_id, shader);
                        }
                        Err(err) => {
                            eprintln!("Error {:#?} creating shader for node: {:#?} {:#?}", err, template, node_id);
                        }
                    }
                }

                if let Some(updater) = NodeUpdate::new(template) {
                    self.updaters.insert(node_id, updater);
                }

                //remove output target if not needed
                for input in node.inputs(self.graph.graph_ref()) {
                    if input.typ == ConnectionType::Texture2D {
                        let connected_output = self.graph.graph_ref().connection(input.id);
                        if let Some(output_id) = connected_output {
                            let connected_node_id = self.graph.graph_ref()[output_id].node;

                            self.output_targets.remove(connected_node_id);
                        }
                    }
                }

                self.add_dangling_output(facade, node_id);
            }

            //may create new output target
            NodeResponse::DisconnectEvent { output: output_id, .. } => {
                if let Some(output) = self.graph.graph_ref().try_get_output(output_id){
                    self.add_dangling_output(facade, output.node);
                }
            }

            NodeResponse::ConnectEventEnded { output, .. } => {
                self.output_targets
                    .remove(self.graph.graph_ref()[output].node);
            }

            NodeResponse::DeleteNodeFull { node_id, .. } => {
                self.output_targets.remove(node_id);
                self.shaders.remove(node_id);
                self.versions.remove(node_id);
                self.updaters.remove(node_id);
                self.node_textures.remove(node_id);
            }
            _ => {}
        }
    }

    ///Processes each shader in the output_targets list from start to end
    /// Generates ui textures
    /// processes inputs
    fn render_shaders(&mut self, facade: &impl Facade, egui_glium: &mut EguiGlium) {
        // let shaders = &mut self.shaders;
        // let graph = &self.graph;

        for (output_id, output_target) in &mut self.output_targets {

            output_target.with_fb_mut(|fb| {
                fb.clear_color(0., 0., 0., 0.);

                self.graph.map_with_inputs(output_id, &mut |node_id, inputs| {

                    // let target = self.texture_manager.get_color(facade);

                    //Render a shader
                    if let Some(shader) = self.shaders.get_mut(node_id) {
                        // let mut surface = target.as_surface();

                        // surface.clear_color(0., 0., 0., 0.);

                        let target = shader.render(facade, &mut self.texture_manager, ShaderInputs::from(&inputs));

                        let surface = target.as_surface();

                        let (w, h) = surface.get_dimensions();
                        let size = (w/4, h/4);

                        // node.user_data.
                        // self.node_textures[node_id].borrow_mut()
                        let mut ui_texture = (*self.node_textures[node_id]).borrow_mut();
                        ui_texture.update_size(facade, egui_glium, size);
                        ui_texture.copy_from(&surface);

                        Some(target)
                    } else {
                        None
                    }
                }, &mut SecondaryMap::new());
            });

            // println!("RENDERED {rendered_node_names} to {}", self.graph[output_id].label);
        }
    }

    pub fn update(&mut self, facade: &impl Facade) {
        for (node_id, updater) in self.updaters.iter_mut() {
            let template = &mut self.graph[node_id].user_data.template;

            let node = &mut self.graph.0.graph.nodes[node_id];
            let inputs: Vec<_> = node.inputs.iter()
                .map(|(name, in_id)| (name.as_str(), &self.graph.0.graph.inputs[*in_id]))
                .collect();

            updater.update(
                facade, 
                &mut node.user_data, 
                &inputs, 
                &mut self.shaders[node_id]
            )
        }
    }

    pub fn draw(&mut self, display: &Display, egui_glium: &mut EguiGlium) {
        let mut frame = display.draw();

        frame.clear_color_and_depth((1., 1., 1., 1.), 0.);

        let mut graph_response = None;

        let _needs_repaint = egui_glium.run(display, |ctx| {
            // ctx.tex_manager()
            graph_response = Some(self.graph.draw(ctx));
        });

        if let Some(response) = graph_response {
            for event in response.node_responses {
                self.node_event(display, egui_glium, event);
            }
        }

        self.render_shaders(display, egui_glium);

        egui_glium.paint(display, &mut frame);

        frame.finish().unwrap();
    }
}