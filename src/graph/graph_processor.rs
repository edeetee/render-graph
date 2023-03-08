use std::{
    rc::Rc, cell::RefCell, time::Instant,
};

use egui_glium::EguiGlium;
use egui_node_graph::{NodeId, InputId, OutputId};
use glium::{
    backend::Facade,
    framebuffer::{RenderBuffer, SimpleFrameBuffer}, Surface, Texture2d,
};

use ouroboros::self_referencing;
use slotmap::{SecondaryMap, SparseSecondaryMap};
use crate::{textures::{TextureManager}, common::animation::UpdateInfo};

use crate::common::{connections::ConnectionType};

use super::{
    def::{self, *,},
    graph::{GraphUtils},
    node_shader::ShaderInputs, node_shader::NodeShader, node_update::{NodeUpdate},
};

// extern crate gl;

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
    // pub graph: ShaderGraph,
    texture_manager: TextureManager,

    output_targets: SparseSecondaryMap<NodeId, OutputTarget>,

    shaders: SecondaryMap<NodeId, NodeShader>,
    updaters: SecondaryMap<NodeId, NodeUpdate>,

    update_info: UpdateTracker
}

pub enum GraphChangeEvent {
    CreatedNode(NodeId),
    DestroyedNode(NodeId),

    Connected{
        output_id: OutputId, 
        input_id: InputId
    },
    Disconnected{
        output_id: OutputId, 
        input_id: InputId
    },
}

impl GraphChangeEvent {
    #[must_use="Use the vec of node responses to load callbacks"]
    pub fn vec_from_graph(graph: &Graph) -> Vec<Self> {
        let new_nodes = graph.nodes.iter()
            .map(|(node_id, ..)| GraphChangeEvent::CreatedNode(node_id));

        let new_connections = graph.connections.iter()
            .map(|(input, output)| GraphChangeEvent::Connected { output_id: *output, input_id: input });

        new_nodes.chain(new_connections).collect()
    }
}

impl ShaderGraphProcessor {
    pub fn new_from_graph(graph: &mut Graph, facade: &impl Facade) -> Self {
        let mut this = Self::default();

        for event in GraphChangeEvent::vec_from_graph(graph) {
            this.graph_event(graph, facade, event);
        }

        this
    }

    fn add_dangling_output(&mut self, facade: &impl Facade, node_id: NodeId) {
        // let is_output_target = node.outputs(&graph.graph_ref()).any(|o| o.typ == NodeConnectionTypes::Texture2D);

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

    ///Call with the response of the graph editor ui to update the slotmaps
    pub fn graph_event(
        &mut self,
        graph: &mut Graph,
        facade: &impl Facade,
        event: GraphChangeEvent,
    ) {
        match event {
            GraphChangeEvent::CreatedNode(node_id) => {
                let template = &graph[node_id].user_data.template;

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
                            graph[node_id].user_data.create_error = Some(err.into());
                            // eprintln!("Error {:#?} creating shader for node: {:#?} {:#?}", err, template, node_id);
                        }
                    }
                }

                let template = &graph[node_id].user_data.template;

                if let Some(updater) = NodeUpdate::new(template) {
                    self.updaters.insert(node_id, updater);
                }

                //remove output target if not needed
                for input in graph[node_id].inputs(graph) {
                    if input.typ == ConnectionType::Texture2D {
                        let connected_output = graph.connection(input.id);
                        if let Some(output_id) = connected_output {
                            let connected_node_id = graph[output_id].node;

                            self.output_targets.remove(connected_node_id);
                        }
                    }
                }

                self.add_dangling_output(facade, node_id);
            }

            //may create new output target
            GraphChangeEvent::Disconnected { output_id, .. } => {
                if let Some(output) = graph.try_get_output(output_id){
                    self.add_dangling_output(facade, output.node);
                }
            }

            GraphChangeEvent::Connected { output_id, .. } => {
                self.output_targets
                    .remove(graph[output_id].node);
            }

            GraphChangeEvent::DestroyedNode (node_id) => {
                self.output_targets.remove(node_id);
                self.shaders.remove(node_id);
                self.updaters.remove(node_id);
            }
        }
    }

    ///Processes each shader in the output_targets list from start to end
    /// Generates ui textures
    /// processes inputs
    pub fn render_shaders(&mut self, graph: &mut Graph, facade: &impl Facade, mut node_post_render: impl FnMut(NodeId, &Texture2d))
     {
        // let shaders = &mut self.shaders;
        // let graph = &graph;

        let mut errors: SparseSecondaryMap<NodeId, NodeError> = Default::default();

        for (output_id, output_target) in &mut self.output_targets {
            output_target.with_fb_mut(|fb| {
                fb.clear_color(0., 0., 0., 0.);

                graph.map_with_inputs(output_id, &mut |node_id, inputs| {

                    //Render a shader
                    if let Some(shader) = self.shaders.get_mut(node_id) {

                        match shader.render(facade, &mut self.texture_manager, ShaderInputs::from(&inputs)) {
                            Ok(target) => {
                                node_post_render(node_id, &target);
                                // facade.

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

        for (node_id, data) in graph.nodes.iter_mut() {
            data.user_data.render_error = errors.remove(node_id);
        }
    }

    pub fn update(&mut self, graph: &mut Graph, state: &GraphState, facade: &impl Facade) {
        for (node_id, updater) in self.updaters.iter_mut() {
            let _template = &mut graph[node_id].user_data.template;

            let node = &mut graph.nodes[node_id];
            let inputs: Vec<_> = node.inputs.iter()
                .map(|(name, in_id)| (name.as_str(), &graph.inputs[*in_id]))
                .collect();

            if let Some(shader) = &mut self.shaders.get_mut(node_id) {
                match updater.update(
                    facade, 
                    &mut node.user_data.template, 
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
        
        for ((node_id, param_name), animation) in &state.animations {
            let maybe_input = graph
                .nodes[*node_id].inputs.iter()
                .find(|(input_name, _)| input_name == param_name);

            if let Some((_, input_id)) = maybe_input {
                let input_id = *input_id;
                let input_param = &mut graph.inputs[input_id].value;
                animation.update_value(input_param, &update_info);
            }
        }

        self.update_info.last_update = Instant::now();
    }

    
}

