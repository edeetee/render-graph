use std::{time::SystemTime, path::PathBuf};

use enum_dispatch::enum_dispatch;
use glium::backend::Facade;
use strum::Display;
use thiserror::Error;

use crate::{isf::{meta::{IsfInfo, IsfInfoReadError}, shader::{IsfShader, IsfShaderLoadError}, updater::IsfUpdater}, obj_shader::loader::ObjLoader, gl_expression::GlExpressionUpdater};

use super::{node_types::NodeType, def::{EditorState, NodeData, UiValue}, node_shader::NodeShader, graph::InputParams};


pub enum NodeUpdate {
    Isf(IsfUpdater),
    Obj(ObjLoader),
    Expression(GlExpressionUpdater)
}

//TODO: Only run on change (ui etc)
//Maybe time to use ECS?

impl NodeUpdate {
    pub fn new(template: &NodeType) -> Option<Self> {
        match template {
            NodeType::Isf { .. } => Some(Self::Isf(IsfUpdater{modified: SystemTime::now()})),
            NodeType::ObjRender => Some(Self::Obj(ObjLoader::new())),
            NodeType::Expression { .. } => Some(Self::Expression(GlExpressionUpdater{frag_source: None})),
            _ => None
        }
    }

    pub fn update(&mut self, facade: &impl Facade, data: &mut NodeData, inputs: &InputParams<'_>, shader: &mut NodeShader) {
        match (self, &mut data.template, shader) {
            (
                NodeUpdate::Isf(updater),
                NodeType::Isf { info: isf_info },
                NodeShader::Isf(shader)
            ) => {
                updater.reload_if_updated(facade, isf_info, shader);
            },
            (
                NodeUpdate::Obj(loader),
                _,
                NodeShader::Obj(obj_renderer)
            ) => {
                if let Some(Some(path)) = inputs.iter().filter_map(|(name, input)| {
                    match &input.value {
                        UiValue::Path(path) => Some(path),
                        _ => None
                    }
                }).next() {
                    loader.load_if_changed(facade, &path, obj_renderer);
                }
            },
            (
                NodeUpdate::Expression(updater),
                NodeType::Expression { .. },
                NodeShader::Expression(renderer)
            ) => {

                if let Some(frag_source) = inputs.iter()
                    .find_map(|(name, val)| {
                        if let UiValue::Text(text, ..) = &val.value {
                            Some(text.value.clone())
                        } else {
                            None
                        }
                    }) 
                {
                    if let Some(inputs) = updater.update(facade, renderer, frag_source) {
                        dbg!(inputs);
                    }
                }
            }
            _ => {}
        }
    }
}