use egui_node_graph::{Graph, NodeId, NodeTemplateIter, NodeTemplateTrait};
use glam::Mat4;
use serde::{Deserialize, Serialize};
use std::fmt::Display;
use std::path::{Path, PathBuf};

use super::def::{GraphState, UiNodeData};

use crate::common::mat4_animator::Mat4Animator;
use shaders::isf::meta::{default_isf_path, IsfInfo};

use crate::common::connections::{ConnectionType, InputDef, OutputDef};
use crate::common::def::{TextStyle, UiValue};

///Enum of node types used to create an actual node
#[derive(Clone, PartialEq, Debug, Serialize, Deserialize)]
pub enum NodeType {
    // #[cfg()]
    SharedOut,
    ObjRender,
    Isf { info: IsfInfo },
    Expression { inputs: Option<Vec<InputDef>> },
}

pub trait GetTemplate {
    fn template(&self) -> &NodeType;
    fn template_mut(&mut self) -> &mut NodeType;
}

impl Display for NodeType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.get_name())
    }
}

pub fn default_node_dirs() -> Vec<PathBuf> {
    vec![
        Path::new(env!("CARGO_MANIFEST_DIR")).join("isf_shaders"),
        default_isf_path(),
    ]
}

impl NodeType {
    pub fn get_name(&self) -> &str {
        match self {
            NodeType::SharedOut => "SpoutOut",
            NodeType::ObjRender => "ObjRender",
            NodeType::Isf { info } => info.name.as_str(),
            NodeType::Expression { .. } => "Expression",
        }
    }

    pub fn try_from_path(path: &Path) -> Option<NodeType> {
        let info = IsfInfo::try_from_path(path).ok()?;

        Some(NodeType::Isf { info })
    }

    pub fn get_builtin() -> Vec<NodeType> {
        // let path = default_isf_path();
        // let shaders = parse_isf_shaders(&path)
        //     .map(|info| NodeType::Isf{info});

        let types = vec![
            // NodeTypes::Instances,
            // NodeTypes::Output,
            NodeType::ObjRender,
            NodeType::SharedOut,
            NodeType::Expression { inputs: None },
        ];
        // types.extend(shaders);

        types
    }

    pub fn get_input_types(&self) -> Vec<InputDef> {
        match self {
            NodeType::Isf { info } => info.def.inputs.iter().map(InputDef::from).collect(),
            NodeType::SharedOut => vec![("name", "RustSpout").into(), InputDef::texture("texture")],
            NodeType::ObjRender => vec![
                ("obj", UiValue::Path(None)).into(),
                ("model", UiValue::Mat4(Mat4::IDENTITY.into())).into(),
                ("view", UiValue::Mat4(Mat4Animator::new_view())).into(),
            ],
            NodeType::Expression { .. } => vec![
                (
                    "text",
                    UiValue::Text(
                        "vec4(1.0,1.0,1.0,1.0)".to_string().into(),
                        TextStyle::Multiline,
                    ),
                )
                    .into(),
                InputDef::texture("pixels"),
            ],
        }
    }

    pub fn get_output_types(&self) -> Vec<OutputDef> {
        match self {
            NodeType::SharedOut => vec![],
            NodeType::Isf { .. } => vec![ConnectionType::Texture2D.into()],
            NodeType::ObjRender => vec![ConnectionType::Texture2D.into()],
            NodeType::Expression { .. } => vec![ConnectionType::Texture2D.into()], // _ => vec![ConnectionType::Texture2D.into()],
        }
    }
}
pub struct AllNodeTypes;
impl NodeTemplateIter for AllNodeTypes {
    type Item = NodeType;

    fn all_kinds(&self) -> Vec<Self::Item> {
        NodeType::get_builtin()
    }
}
