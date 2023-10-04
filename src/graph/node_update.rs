use super::def::GraphState;
use super::graph_change_listener::{GraphChangeEvent, GraphUpdateListener};
use super::{graph_utils::InputParams, node_shader::NodeShader, node_types::NodeType};
use crate::common::def::UiValue;
use crate::graph::def::Graph;
use crate::{
    gl_expression::GlExpressionUpdater, isf::updater::IsfUpdater, obj_shader::loader::ObjLoader,
};
use egui_node_graph::NodeId;
use glium::backend::Facade;
use slotmap::SecondaryMap;
use std::time::{Instant, SystemTime};

pub struct UpdateTracker {
    last_update: Instant,
}

impl Default for UpdateTracker {
    fn default() -> Self {
        Self {
            last_update: Instant::now(),
        }
    }
}

#[derive(Default)]
pub struct NodeUpdaters {
    update_info: UpdateTracker,
    updaters: SecondaryMap<NodeId, UpdateShader>,
}

impl NodeUpdaters {
    pub fn update(
        &mut self,
        shaders: &mut SecondaryMap<NodeId, NodeShader>,
        graph: &mut Graph,
        facade: &impl Facade,
    ) {
        for (node_id, updater) in self.updaters.iter_mut() {
            let _template = &mut graph[node_id].user_data.template;

            let node = &mut graph.nodes[node_id];
            let inputs: Vec<_> = node
                .inputs
                .iter()
                .map(|(name, in_id)| (name.as_str(), &graph.inputs[*in_id]))
                .collect();

            if let Some(shader) = shaders.get_mut(node_id) {
                match updater.update(facade, &mut node.user_data.template, &inputs, shader) {
                    Ok(()) => node.user_data.update_error = None,
                    Err(err) => node.user_data.update_error = Some(err.into()),
                }
            }
        }
    }
}

impl GraphUpdateListener for NodeUpdaters {
    fn graph_event(&mut self, graph: &mut Graph, facade: &impl Facade, event: GraphChangeEvent) {
        match event {
            GraphChangeEvent::CreatedNode(node_id) => {
                let template = &graph[node_id].user_data.template;

                if let Some(updater) = UpdateShader::new(template) {
                    self.updaters.insert(node_id, updater);
                }
            }

            GraphChangeEvent::DestroyedNode(node_id) => {
                self.updaters.remove(node_id);
            }

            GraphChangeEvent::Connected { .. } | GraphChangeEvent::Disconnected { .. } => {}
        }
    }
}

pub enum UpdateShader {
    Isf(IsfUpdater),
    Obj(ObjLoader),
    Expression(GlExpressionUpdater),
}

//TODO: Only run on change (ui etc)
//Maybe time to use ECS?

impl UpdateShader {
    pub fn new(template: &NodeType) -> Option<Self> {
        match template {
            NodeType::Isf { .. } => Some(Self::Isf(IsfUpdater {
                modified: SystemTime::now(),
            })),
            NodeType::ObjRender => Some(Self::Obj(ObjLoader::new())),
            NodeType::Expression { .. } => {
                Some(Self::Expression(GlExpressionUpdater { frag_source: None }))
            }
            _ => None,
        }
    }

    pub fn update(
        &mut self,
        facade: &impl Facade,
        template: &mut NodeType,
        inputs: &InputParams<'_>,
        shader: &mut NodeShader,
    ) -> anyhow::Result<()> {
        match (self, template, shader) {
            (
                UpdateShader::Isf(updater),
                NodeType::Isf { info: isf_info },
                NodeShader::Isf(shader),
            ) => {
                updater.reload_if_updated(facade, isf_info, shader)?;
            }

            (UpdateShader::Obj(loader), _, NodeShader::Obj(obj_renderer)) => {
                if let Some(Some(path)) =
                    inputs.iter().find_map(|(_name, input)| match &input.value {
                        UiValue::Path(path) => Some(path),
                        _ => None,
                    })
                {
                    loader.load_if_changed(facade, &path, obj_renderer)?;
                }
            }

            (
                UpdateShader::Expression(updater),
                NodeType::Expression { .. },
                NodeShader::Expression(renderer),
            ) => {
                if let Some(frag_source) = inputs.iter().find_map(|(_name, val)| {
                    if let UiValue::Text(text, ..) = &val.value {
                        Some(text.value.clone())
                    } else {
                        None
                    }
                }) {
                    let _inputs = updater.update(facade, renderer, frag_source)?;
                    // dbg!(inputs);
                }
            }
            _ => {}
        }

        Ok(())
    }
}
