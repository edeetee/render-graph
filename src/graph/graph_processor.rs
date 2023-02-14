use std::{
    rc::Rc, cell::RefCell, time::Instant, default,
};

use egui_glium::EguiGlium;
use egui_node_graph::{NodeId, NodeResponse};
use glium::{
    backend::Facade,
    Display,
    framebuffer::{RenderBuffer, SimpleFrameBuffer}, Surface,
};

use ouroboros::self_referencing;
use slotmap::{SecondaryMap, SparseSecondaryMap};
use crate::{textures::{UiTexture, TextureManager}, common::animation::UpdateInfo};

use crate::common::{def::*, connections::ConnectionType};

use super::{
    def::{self, *},
    graph::ShaderGraph,
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

pub struct UpdateTracker {
    last_update: Instant
}
impl Default for UpdateTracker {
    fn default() -> Self {
        Self { last_update: Instant::now() }
    }
}

#[derive(Default)]
pub struct ShaderGraphProcessor {
    pub graph: ShaderGraph,
    texture_manager: TextureManager,

    output_targets: SparseSecondaryMap<NodeId, OutputTarget>,
    node_textures: SecondaryMap<NodeId, Rc<RefCell<UiTexture>>>,

    shaders: SecondaryMap<NodeId, NodeShader>,
    updaters: SecondaryMap<NodeId, NodeUpdate>,

    update_info: UpdateTracker
}

impl ShaderGraphProcessor {

    pub fn new(graph: ShaderGraph) -> ShaderGraphProcessor {
        Self {
            graph,
            ..Default::default()
        }
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
                let ui_texture = UiTexture::new(facade, egui_glium, (256, 256));
                let textures = Rc::new(RefCell::new(ui_texture));
                self.graph[node_id].user_data.texture = Rc::downgrade(&textures);
                self.node_textures.insert(node_id, textures);

                let template = &self.graph[node_id].user_data.template;

                //only add if needed ()s
                if let Some(shader) = NodeShader::new(template, facade) {
                    match shader {
                        Ok(shader) => {
                            self.shaders.insert(node_id, shader);

                            if let Some(updater) = NodeUpdate::new(template) {
                                self.updaters.insert(node_id, updater);
                            }
                        }
                        Err(err) => {
                            self.graph[node_id].user_data.create_error = Some(err.into());
                            // eprintln!("Error {:#?} creating shader for node: {:#?} {:#?}", err, template, node_id);
                        }
                    }
                }

                let template = &self.graph[node_id].user_data.template;

                if let Some(updater) = NodeUpdate::new(template) {
                    self.updaters.insert(node_id, updater);
                }

                //remove output target if not needed
                for input in self.graph[node_id].inputs(self.graph.graph_ref()) {
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

        let mut errors: SparseSecondaryMap<NodeId, NodeError> = Default::default();

        for (output_id, output_target) in &mut self.output_targets {

            output_target.with_fb_mut(|fb| {
                fb.clear_color(0., 0., 0., 0.);

                self.graph.map_with_inputs(output_id, &mut |node_id, inputs| {

                    // let target = self.texture_manager.get_color(facade);

                    //Render a shader
                    if let Some(shader) = self.shaders.get_mut(node_id) {
                        // let mut surface = target.as_surface();


                        let mut ui_texture = (*self.node_textures[node_id]).borrow_mut();

                        match shader.render(facade, &mut self.texture_manager, ShaderInputs::from(&inputs)) {
                            Ok(target) => {
                                let surface = target.as_surface();
                                // surface.clear_color(0., 0., 0., 0.);

                                let (w, h) = surface.get_dimensions();
                                let size = (w, h);
        
                                // node.user_data.
                                // self.node_textures[node_id].borrow_mut()
                                ui_texture.update_size(facade, egui_glium, size);
                                ui_texture.copy_from(&surface);

                                Some(target)
                            }

                            Err(err) => {
                                errors.insert(node_id, err.into());
                                None
                            }
                        }
                    } else {
                        None
                    }
                }, &mut SecondaryMap::new());
            });
        }

        for (node_id, data) in self.graph.editor.graph.nodes.iter_mut() {
            data.user_data.render_error = errors.remove(node_id);
        }
    }

    pub fn update(&mut self, facade: &impl Facade) {
        for (node_id, updater) in self.updaters.iter_mut() {
            let _template = &mut self.graph[node_id].user_data.template;

            let node = &mut self.graph.editor.graph.nodes[node_id];
            let inputs: Vec<_> = node.inputs.iter()
                .map(|(name, in_id)| (name.as_str(), &self.graph.editor.graph.inputs[*in_id]))
                .collect();

            if let Some(shader) = &mut self.shaders.get_mut(node_id) {
                match updater.update(
                    facade, 
                    &mut node.user_data, 
                    &inputs, 
                    shader
                ) {
                    Ok(()) => {
                        node.user_data.update_error = None
                    },
                    Err(err) => {
                        node.user_data.update_error = Some(err.into())
                    }
                }
            }
        }

        let elapsed_since_update = self.update_info.last_update.elapsed();
        let update_info = UpdateInfo::new(elapsed_since_update);
        
        for ((node_id, param_name), animation) in &self.graph.state.animations {
            let maybe_input = self.graph.graph_ref()
                .nodes[*node_id].inputs.iter()
                .find(|(input_name, _)| input_name == param_name);

            if let Some((_, input_id)) = maybe_input {
                let input_id = *input_id;
                let input_param = &mut self.graph.editor.graph.inputs[input_id].value;
                animation.update_value(input_param, &update_info);
            }
        }

        self.update_info.last_update = Instant::now();
    }

    pub fn draw(&mut self, display: &Display, egui_glium: &mut EguiGlium) {
        let mut frame = display.draw();

        frame.clear_color_and_depth((1., 1., 1., 1.), 0.);

        let mut graph_response = None;

        let _needs_repaint = egui_glium.run(display, |ctx| {
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