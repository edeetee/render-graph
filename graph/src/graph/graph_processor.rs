use std::{
    collections::HashSet,
    rc::Rc,
    time::{Duration, Instant},
};

use egui_node_graph::{NodeId};
use glium::{backend::Facade, Texture2d};

use crate::{
    def::{AsUniformOptional, GetUiValue},
    textures::TextureManager,
    GetTemplate,
};
use itertools::Itertools;
use slotmap::{SecondaryMap, SparseSecondaryMap};

use crate::common::connections::ConnectionType;

use super::{
    def::*,
    graph_change_listener::{GraphChangeEvent, GraphUpdateListener},
    graph_utils::GraphMap,
    node_shader::NodeShader,
    node_shader::ProcessedShaderNodeInputs,
    node_update::{NodeUpdaters},
};

#[derive(Default)]
pub struct GraphShaderProcessor {
    terminating_nodes: HashSet<NodeId>,
    shaders: SecondaryMap<NodeId, NodeShader>,
    updater: NodeUpdaters,
}

impl std::fmt::Debug for GraphShaderProcessor {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ShaderGraphProcessor")
            .field("terminating_nodes", &self.terminating_nodes)
            .field("shaders", &self.shaders.len())
            .field("updater", &stringify!(NodeUpdaters))
            .finish()
    }
}

pub struct RenderResponse {
    pub terminating_textures: Vec<Option<Rc<Texture2d>>>,
    pub errors: SparseSecondaryMap<NodeId, NodeError>,
    pub times: SecondaryMap<NodeId, Duration>,
}

impl GraphShaderProcessor {
    fn add_dangling_output(&mut self, _facade: &impl Facade, node_id: NodeId) {
        self.terminating_nodes.insert(node_id);
    }

    ///Processes each shader in the output_targets list from start to end
    /// Generates ui textures
    /// processes inputs
    /// Returns a list of output textures
    pub fn render_shaders<'a, N, C, V: AsUniformOptional>(
        &mut self,
        graph: &mut egui_node_graph::Graph<N, C, V>,
        facade: &impl Facade,
        texture_manager: &mut TextureManager,
        mut node_post_render: impl FnMut(NodeId, &Texture2d),
    ) -> RenderResponse {
        let mut errors: SparseSecondaryMap<NodeId, NodeError> = Default::default();
        let mut times: SecondaryMap<NodeId, Duration> = Default::default();

        let outputs = self
            .terminating_nodes
            .iter()
            .cloned()
            .map(|output_id| {
                graph.map_with_inputs(
                    output_id,
                    &mut |node_id, inputs| {
                        //Render a shader
                        let start = Instant::now();
                        if let Some(shader) = self.shaders.get_mut(node_id) {
                            match shader.render(
                                facade,
                                texture_manager,
                                ProcessedShaderNodeInputs::from(&inputs),
                            ) {
                                Ok(target) => {
                                    node_post_render(node_id, &target);
                                    times.insert(node_id, start.elapsed());
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
                    },
                    &mut SecondaryMap::new(),
                )
            })
            .collect();

        RenderResponse {
            terminating_textures: outputs,
            errors,
            times,
        }
    }
}

impl<N: GetTemplate, V> GraphUpdateListener<N, ConnectionType, V> for GraphShaderProcessor {
    ///Call with the response of the graph editor ui to update the slotmaps
    fn graph_event(
        &mut self,
        graph: &mut egui_node_graph::Graph<N, ConnectionType, V>,
        facade: &impl Facade,
        event: GraphChangeEvent,
    ) -> anyhow::Result<()> {
        match event {
            GraphChangeEvent::CreatedNode(node_id) => {
                let template = graph[node_id].user_data.template();

                //only add if needed ()s
                if let Some(shader) = NodeShader::new(template, facade) {
                    self.shaders.insert(node_id, shader?);
                }

                //remove output target if not needed
                for input in graph[node_id].inputs(graph) {
                    if input.typ == ConnectionType::Texture2D {
                        let connected_output = graph.connection(input.id);
                        if let Some(output_id) = connected_output {
                            let connected_node_id = graph[output_id].node;

                            self.terminating_nodes.remove(&connected_node_id);
                        }
                    }
                }

                self.add_dangling_output(facade, node_id);
            }

            //may create new output target
            GraphChangeEvent::Disconnected { output_id, .. } => {
                if let Some(output) = graph.try_get_output(output_id) {
                    self.add_dangling_output(facade, output.node);
                }
            }

            GraphChangeEvent::Connected { output_id, .. } => {
                self.terminating_nodes.remove(&graph[output_id].node);
            }

            GraphChangeEvent::DestroyedNode(node_id) => {
                self.terminating_nodes.remove(&node_id);
                self.shaders.remove(node_id);
            }
        }

        self.updater.graph_event(graph, facade, event);

        Ok(())
    }
}

impl GraphShaderProcessor {
    pub fn update<N: GetTemplate, C, V: GetUiValue>(
        &mut self,
        graph: &mut egui_node_graph::Graph<N, C, V>,
        facade: &impl Facade,
    ) -> SparseSecondaryMap<NodeId, anyhow::Error> {
        self.updater.update(&mut self.shaders, graph, facade)
    }
}
