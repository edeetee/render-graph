use crate::{
    graph::{
        animator::Animator,
        def::{Graph, GraphEditorState, GraphState, UniqueNodeName},
        MultipleUpdatesListener,
    },
    util::{read_from_json_file, write_to_json_file},
};
use egui_node_graph::NodeId;
use serde::{Deserialize, Serialize};
use std::{collections::HashMap, env, path::PathBuf};

#[derive(Default, Serialize, Deserialize)]
pub struct PersistentState {
    pub graph_editor: GraphEditorState,
    pub animator: Animator,

    #[serde(with = "vectorize")]
    pub node_names: HashMap<NodeId, UniqueNodeName>,

    pub window: Option<WindowState>,

    #[cfg(feature = "editor")]
    pub graph_ui_state: Option<crate::editor::graph_ui::GraphUiState>,
}

impl PersistentState {
    pub fn default_path() -> PathBuf {
        PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("render-graph-auto-save.json")
    }

    pub fn load_from_default_path() -> Self {
        Self::load_from_file_or_default(&Self::default_path())
    }

    fn load_from_file_or_default(file: &PathBuf) -> Self {
        match read_from_json_file::<Self>(file) {
            Ok(state) => {
                println!("Loaded save file from {file:?}");
                state
            }
            Err(err) => {
                eprintln!("Failed to read default save {file:?}\nERR({err:?}). Using new graph");
                Self::default()
            }
        }
    }

    pub fn write_to_default_path(self) -> anyhow::Result<()> {
        write_to_json_file(&Self::default_path(), &self)
    }
}

#[derive(Default, Serialize, Deserialize, Clone, Copy)]
pub struct WindowState {
    pub res: (u32, u32),
    pub fullscreen: bool,
}
