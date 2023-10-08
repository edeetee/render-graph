use egui_node_graph::NodeId;
use graph::{
    connections::ConnectionType, def::UiValue, Animator, Graph, GraphEditorState, UiNodeData,
    UniqueNodeName,
};

pub mod ui_state;

use serde::{de::DeserializeOwned, Deserialize, Serialize};
use slotmap::SecondaryMap;
use std::{
    collections::HashMap,
    env,
    fs::File,
    path::{Path, PathBuf},
};

#[derive(Serialize, Deserialize)]
pub struct PersistentState<N = UiNodeData, C = ConnectionType, V = UiValue> {
    pub graph: egui_node_graph::Graph<N, C, V>,
    pub node_positions: SecondaryMap<NodeId, egui::Pos2>,

    pub animator: Animator,

    pub node_names: SecondaryMap<NodeId, UniqueNodeName>,

    pub window: Option<WindowState>,

    pub graph_ui_state: Option<ui_state::GraphUiState>,
}

impl<N, C, V> Default for PersistentState<N, C, V> {
    fn default() -> Self {
        Self {
            graph: Default::default(),
            node_positions: Default::default(),
            animator: Default::default(),
            node_names: Default::default(),
            window: Default::default(),
            graph_ui_state: Default::default(),
        }
    }
}

impl<N, C, V> PersistentState<N, C, V> {
    pub fn default_path() -> PathBuf {
        PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("render-graph-auto-save.json")
    }
}

impl<
        N: Serialize + DeserializeOwned,
        C: Serialize + DeserializeOwned,
        V: Serialize + DeserializeOwned,
    > PersistentState<N, C, V>
{
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

pub fn write_to_json_file(path: &Path, data: &impl Serialize) -> anyhow::Result<()> {
    let file = File::create(path)?;
    serde_json::to_writer_pretty(file, data)?;

    Ok(())
}

// pub fn write_to_toml_file(path: &Path, data: &impl Serialize) -> anyhow::Result<()> {
//     let mut file = File::create(path)?;
//     let mut output = String::new();
//     data.serialize(toml::Serializer::pretty(&mut output))?;
//     file.write_all(output.as_bytes())?;

//     Ok(())
// }

pub fn read_from_json_file<T: DeserializeOwned>(path: &Path) -> anyhow::Result<T> {
    let file = File::open(path)?;
    Ok(serde_json::from_reader(file)?)
}
