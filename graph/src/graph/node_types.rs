use common::files::map_file_tree;
use common::tree::{BranchIndex, LeafIndex, Tree, TreeStructure};
use egui_node_graph::{NodeTemplateIter};
use glam::Mat4;
use serde::{Deserialize, Serialize};
use slotmap::SlotMap;
use std::fmt::Display;
use std::path::{Path, PathBuf};

use crate::common::mat4_animator::Mat4Animator;
use shaders::isf::meta::{default_isf_path, IsfInfo};

use crate::common::connections::{ConnectionType, InputDef, OutputDef};
use crate::common::def::{TextStyle, UiValue};

///Enum of node types used to create an actual node
#[derive(Clone, PartialEq, Debug, Serialize, Deserialize)]
pub enum NodeType {
    SharedOut,
    ObjRender,
    Isf {
        info: IsfInfo,
    },
    Expression {
        inputs: Option<Vec<InputDef>>,
        name: String,
        source: String,
    },
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

impl NodeType {
    pub fn get_name(&self) -> &str {
        match self {
            NodeType::SharedOut => "SpoutOut",
            NodeType::ObjRender => "ObjRender",
            NodeType::Isf { info } => info.name.as_str(),
            NodeType::Expression { name, .. } => {
                if name.is_empty() {
                    &"Expression"
                } else {
                    &name
                }
            }
        }
    }

    pub fn try_from_path(path: &Path) -> Option<NodeType> {
        let info = IsfInfo::try_from_path(path).ok()?;

        Some(NodeType::Isf { info })
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
            NodeType::Expression { source, .. } => vec![
                (
                    "text",
                    UiValue::Text(
                        if source.is_empty() {
                            "vec4(1.0,1.0,1.0,1.0)".to_string()
                        } else {
                            source.clone()
                        }
                        .into(),
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

pub fn default_isf_dirs() -> Vec<PathBuf> {
    vec![
        Path::new(env!("CARGO_MANIFEST_DIR")).join("isf_shaders"),
        default_isf_path(),
    ]
}

impl NodeType {
    pub fn templates() -> Tree<String, NodeType> {
        let isf_paths = default_isf_dirs();

        let mut branches: SlotMap<BranchIndex, String> = SlotMap::default();
        let mut leaves: SlotMap<LeafIndex, NodeType> = SlotMap::default();

        let isf_templates = isf_paths
            .into_iter()
            .flat_map(|path| {
                map_file_tree(
                    path,
                    &mut |path, files| {
                        Some(TreeStructure::Branch(
                            branches
                                .insert(path.file_name().unwrap().to_str().unwrap().to_string()),
                            files.into_iter().filter_map(|f| f).collect(),
                        ))
                    },
                    &mut |file| {
                        NodeType::try_from_path(&file)
                            .map(|node_ty| TreeStructure::Leaf(leaves.insert(node_ty)))
                    },
                )
            })
            .collect();

        let expression_templates = vec![
            ("white", "vec4(1.0,1.0,1.0,1.0)"),
            ("dot", "vec4(vec3(dot(pixel.rgb, vec3(1,0,0))), pixel.a)"),
            ("mod", "vec4(mod(pixel.rgb, vec3(0.1))/vec3(0.1),pixel.a)"),
        ];

        let expressions = expression_templates
            .into_iter()
            .map(|(name, text)| {
                TreeStructure::Leaf(leaves.insert(NodeType::Expression {
                    inputs: None,
                    source: text.to_string(),
                    name: name.to_string(),
                }))
            })
            .collect();

        let defaults = Self::defaults()
            .into_iter()
            .map(|node| TreeStructure::Leaf(leaves.insert(node)))
            .collect();

        Tree {
            tree: vec![
                TreeStructure::Branch(branches.insert("defaults".to_string()), defaults),
                TreeStructure::Branch(branches.insert("isf".to_string()), isf_templates),
                TreeStructure::Branch(branches.insert("expressions".to_string()), expressions),
            ],
            branches,
            leaves,
        }
    }

    pub fn defaults() -> Vec<NodeType> {
        let types = vec![
            NodeType::ObjRender,
            NodeType::SharedOut,
            NodeType::Expression {
                inputs: None,
                name: String::default(),
                source: String::default(),
            },
        ];

        types
    }
}

pub struct AllNodeTypes;
impl NodeTemplateIter for AllNodeTypes {
    type Item = NodeType;

    fn all_kinds(&self) -> Vec<Self::Item> {
        NodeType::defaults()
    }
}
