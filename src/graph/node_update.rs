use std::{time::SystemTime};


use glium::backend::Facade;



use crate::{isf::{updater::IsfUpdater}, obj_shader::loader::ObjLoader, gl_expression::GlExpressionUpdater};

use super::{node_types::NodeType, def::{NodeData, UiValue}, node_shader::NodeShader, graph::InputParams};


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
                if let Some(Some(path)) = inputs.iter().find_map(|(_name, input)| {
                    match &input.value {
                        UiValue::Path(path) => Some(path),
                        _ => None
                    }
                }) {
                    loader.load_if_changed(facade, &path, obj_renderer);
                }
            },
            
            (
                NodeUpdate::Expression(updater),
                NodeType::Expression { .. },
                NodeShader::Expression(renderer)
            ) => {

                if let Some(frag_source) = inputs.iter()
                    .find_map(|(_name, val)| {
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