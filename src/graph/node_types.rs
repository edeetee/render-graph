use std::{fmt::Display};
use egui_node_graph::{NodeTemplateTrait, Graph, NodeId, NodeTemplateIter};
use glam::Mat4;
use serde::{Serialize, Deserialize};

use super::def::{NodeData, GraphState};

use crate::isf::meta::{IsfInfo};

use crate::common::def::{ConnectionType, Mat4UiData, TextStyle, UiValue};
use crate::common::node_connections::{InputDef, OutputDef};
// use crate::common::node_

///Enum of node types used to create an actual node
#[derive(Clone, PartialEq, Debug, Serialize, Deserialize)]
pub enum NodeType {
    // #[cfg()]
    SharedOut,
    ObjRender,
    Isf {
        info: IsfInfo
    },
    Expression {
        inputs: Option<Vec<InputDef>>
    }
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
            NodeType::Isf{info} => info.name.as_str(),
            NodeType::Expression { .. } => "Expression"
        }
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
            NodeType::Expression { inputs: None }
        ];
        // types.extend(shaders);

        types
    }

    pub fn get_input_types(&self) -> Vec<InputDef> {
        match self {
            NodeType::Isf { info } => {
                info.def.inputs.iter().map(InputDef::from).collect()
            }
            NodeType::SharedOut => vec![
                ("name", "RustSpout").into(),
                InputDef::texture("texture"),
            ],
            NodeType::ObjRender => vec![
                ("obj", UiValue::Path(None)).into(),
                ("model", UiValue::Mat4(Mat4::IDENTITY.into())).into(),
                ("view", UiValue::Mat4(Mat4UiData::new_view())).into(),
            ],
            NodeType::Expression { .. } => vec![
                ("text", UiValue::Text("vec4(1.0,1.0,1.0,1.0)".to_string().into(), TextStyle::Multiline)).into(),
                InputDef::texture("pixels")
            ]
        }
    }

    pub fn get_output_types(&self) -> Vec<OutputDef> {
        match self {
            NodeType::SharedOut => vec![],
            NodeType::Isf { .. } => vec![ConnectionType::Texture2D.into()],
            NodeType::ObjRender => vec![ConnectionType::Texture2D.into()],
            NodeType::Expression { .. } => vec![ConnectionType::Texture2D.into()]
            // _ => vec![ConnectionType::Texture2D.into()],
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

impl NodeTemplateTrait for NodeType {
    type NodeData = NodeData;
    type DataType = ConnectionType;
    type ValueType = UiValue;
    type UserState = GraphState;

    fn node_finder_label(&self, _user_state: &mut Self::UserState) -> std::borrow::Cow<str> {
        self.get_name().into()
    }

    fn node_graph_label(&self, user_state: &mut Self::UserState) -> String {
        self.node_finder_label(user_state).into()
    }

    fn user_data(&self, _user_state: &mut Self::UserState) -> Self::NodeData {
        NodeData::new(self.clone())
    }

    fn build_node(
        &self,
        graph: &mut Graph<Self::NodeData, Self::DataType, Self::ValueType>,
        _user_state: &mut Self::UserState,
        node_id: NodeId
    ) {
        for input in self.get_input_types() {
            let connection = input.ty != ConnectionType::None;
            let value = input.value != UiValue::None;

            let kind = match (connection, value) {
                (true, true) => egui_node_graph::InputParamKind::ConnectionOrConstant,
                (true, false) => egui_node_graph::InputParamKind::ConnectionOnly,
                (false, true) => egui_node_graph::InputParamKind::ConstantOnly,
                (false, false) => continue
            };

            graph.add_input_param(node_id, input.name, input.ty, input.value, kind, true);
        }

        for output in self.get_output_types() {
            graph.add_output_param(node_id, output.name, output.ty);
        }
    }

}