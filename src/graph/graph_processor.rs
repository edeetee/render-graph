use std::{
    collections::HashSet,
    rc::Rc,
    time::{Duration, Instant},
};

use egui_node_graph::{InputId, NodeId, OutputId};
use glium::{backend::Facade, Texture2d};

use crate::{common::animation::UpdateInfo, textures::TextureManager};
use itertools::Itertools;
use slotmap::{SecondaryMap, SparseSecondaryMap};

use crate::common::connections::ConnectionType;

use super::{
    def::*,
    graph_change_listener::{GraphChangeEvent, GraphUpdateListener, GraphUpdater},
    graph_utils::GraphMap,
    node_shader::NodeShader,
    node_shader::ProcessedShaderNodeInputs,
    node_update::{NodeUpdaters, UpdateShader},
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

impl GraphShaderProcessor {
    fn add_dangling_output(&mut self, _facade: &impl Facade, node_id: NodeId) {
        self.terminating_nodes.insert(node_id);
    }

    ///Processes each shader in the output_targets list from start to end
    /// Generates ui textures
    /// processes inputs
    /// Returns a list of output textures
    pub fn render_shaders(
        &mut self,
        graph: &mut Graph,
        facade: &impl Facade,
        texture_manager: &mut TextureManager,
        mut node_post_render: impl FnMut(NodeId, &Texture2d),
    ) -> Vec<Option<Rc<Texture2d>>> {
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
            .collect_vec();

        for (node_id, data) in graph.nodes.iter_mut() {
            data.user_data.render_error = errors.remove(node_id);

            if let Some(time) = times.remove(node_id) {
                data.user_data.update_time_smoothed(time);
            }
        }

        outputs
    }
}

impl GraphUpdateListener for GraphShaderProcessor {
    ///Call with the response of the graph editor ui to update the slotmaps
    fn graph_event(&mut self, graph: &mut Graph, facade: &impl Facade, event: GraphChangeEvent) {
        match event {
            GraphChangeEvent::CreatedNode(node_id) => {
                let template = &graph[node_id].user_data.template;

                //only add if needed ()s
                if let Some(shader) = NodeShader::new(template, facade) {
                    match shader {
                        Ok(shader) => {
                            self.shaders.insert(node_id, shader);
                        }
                        Err(err) => {
                            graph[node_id].user_data.create_error = Some(err.into());
                            // eprintln!("Error {:#?} creating shader for node: {:#?} {:#?}", err, template, node_id);
                        }
                    }
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
    }
}

impl GraphUpdater for GraphShaderProcessor {
    fn update(&mut self, graph: &mut Graph, facade: &impl Facade) {
        self.updater.update(&mut self.shaders, graph, facade);
    }
}
