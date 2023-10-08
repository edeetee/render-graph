use super::def::GraphState;
use super::graph_change_listener::{GraphChangeEvent, GraphUpdateListener};
use super::{graph_utils::InputParams, node_shader::NodeShader, node_types::NodeType};
use crate::common::def::UiValue;
use crate::connections::ConnectionType;
use crate::def::GetUiValue;
use crate::graph::def::Graph;
use crate::GetTemplate;
use egui_node_graph::NodeId;
use glium::backend::Facade;
use shaders::{
    gl_expression::GlExpressionUpdater, isf::updater::IsfUpdater, obj_shader::loader::ObjLoader,
};
use slotmap::{SecondaryMap, SparseSecondaryMap};
use std::time::{Instant, SystemTime};

#[derive(Default)]
pub struct NodeUpdaters {
    updaters: SecondaryMap<NodeId, UpdateShader>,
}

impl NodeUpdaters {
    pub fn update<N: GetTemplate, C, V: GetUiValue>(
        &mut self,
        shaders: &mut SecondaryMap<NodeId, NodeShader>,
        graph: &mut egui_node_graph::Graph<N, C, V>,
        facade: &impl Facade,
    ) -> SparseSecondaryMap<NodeId, anyhow::Error> {
        let mut errors = SparseSecondaryMap::default();

        for (node_id, updater) in self.updaters.iter_mut() {
            let node = &mut graph.nodes[node_id];
            let inputs: Vec<_> = node
                .inputs
                .iter()
                .map(|(name, in_id)| (name.as_str(), &graph.inputs[*in_id]))
                .collect();

            if let Some(shader) = shaders.get_mut(node_id) {
                if let Err(err) =
                    updater.update(facade, node.user_data.template_mut(), &inputs, shader)
                {
                    errors.insert(node_id, err);
                }
            }
        }
        errors
    }
}

impl<N: GetTemplate, C, V> GraphUpdateListener<N, C, V> for NodeUpdaters {
    fn graph_event(
        &mut self,
        graph: &mut egui_node_graph::Graph<N, C, V>,
        facade: &impl Facade,
        event: GraphChangeEvent,
    ) -> anyhow::Result<()> {
        match event {
            GraphChangeEvent::CreatedNode(node_id) => {
                let template = graph[node_id].user_data.template();

                if let Some(updater) = UpdateShader::new(template) {
                    self.updaters.insert(node_id, updater);
                }
            }

            GraphChangeEvent::DestroyedNode(node_id) => {
                self.updaters.remove(node_id);
            }

            GraphChangeEvent::Connected { .. } | GraphChangeEvent::Disconnected { .. } => {}
        }

        Ok(())
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

    pub fn update<C, V: GetUiValue>(
        &mut self,
        facade: &impl Facade,
        template: &mut NodeType,
        inputs: &InputParams<'_, C, V>,
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
                    inputs
                        .iter()
                        .find_map(|(_name, input)| match &input.value.ui_value() {
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
                    if let UiValue::Text(text, ..) = &val.value.ui_value() {
                        Some(text.value.clone())
                    } else {
                        None
                    }
                }) {
                    let _inputs = updater.update(facade, renderer, frag_source)?;
                }
            }
            _ => {}
        }

        Ok(())
    }
}
