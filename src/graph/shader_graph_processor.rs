use std::{
    rc::Rc, time::SystemTime, borrow::BorrowMut, cell::RefCell,
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
    node_types::NodeTypes,
    node_shader::ComputedInputs, node_shader::NodeShader,
};

use crate::isf::shader::reload_ifs_shader;

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
                    NodeTypes::Isf { info } => {
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

                self.graph.map_with_inputs::<_, Rc<Texture2d>>(output_id, &mut |node_id, inputs| {

                    // let textures = self.texture_manager.input_textures_iter(facade, target);
                    
                    let processed_inputs: ComputedInputs = inputs.iter()
                        .filter_map(|(name,node,maybe_process_input)| {
                            //first try process node inputs
                            let value = maybe_process_input.as_ref().map(|process_input| {
                                process_input.as_uniform_value()
                            //else use the value
                            }).or_else(|| {
                                node.value.as_shader_input()
                            });

                            //structure into named uniform values
                            value.map(|value| {
                                (*name, value)
                            })
                            
                        })
                        .collect();

                    let target = self.texture_manager.new_target(facade);

                    //Render a shader
                    if let Some(shader) = self.shaders.get_mut(node_id) {
                        let mut surface = target.as_surface();

                        surface.clear_color(0., 0., 0., 0.);

                        shader.draw(&target, &processed_inputs);

                        let (w, h) = surface.get_dimensions();
                        let size = (w/4, h/4);

                        // node.user_data.
                        // self.node_textures[node_id].borrow_mut()
                        let mut texture = (*self.node_textures[node_id]).borrow_mut();
                        texture.update_size(facade, egui_glium, size);
                        texture.copy_from(&surface);
                    }

                    target
                }, &mut SecondaryMap::new());
            });

            // println!("RENDERED {rendered_node_names} to {}", self.graph[output_id].label);
        }
    }

    pub fn reload_ifs_shaders(&mut self, facade: &impl Facade) {
        for (node_id, version) in self.versions.iter_mut() {
            let template = &mut self.graph[node_id].user_data.template;

            if let NodeTypes::Isf { info } = template {
                let new_version = info.path.metadata().unwrap().modified().unwrap();
                let diff = new_version.duration_since(*version);

                if let Ok(diff) = diff {
                    if 10 < diff.as_millis() {
                        //iterate version even on error (wait for change to retry)
                        *version = new_version;

                        let name = info.name.clone();

                        match reload_ifs_shader(facade, &info) {
                            Ok((new_info, new_shader)) => {
                                self.shaders.insert(node_id, NodeShader::Isf(new_shader));
                                *info = new_info;
                                println!("Reloaded shader: {}", name);
                            }
                            Err(err) => {
                                let err_txt = format!("{:#?}", err);
                                let err_txt = err_txt.replace("\\n", "\n");
                                eprintln!("Error reloading shader {}: {}", info.name, err_txt);
                            }
                        }
                    }
                }
            }
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