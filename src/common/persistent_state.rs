use crate::{
    graph::{
        def::{GraphEditorState, GraphState},
        MultipleUpdatesListener,
    },
    util::{read_from_json_file, write_to_json_file},
};
use serde::{Deserialize, Serialize};
use std::{env, path::PathBuf};

#[derive(Default, Serialize, Deserialize, Clone)]
#[serde(default)]
pub struct PersistentState {
    pub editor: GraphEditorState,
    state: GraphState,
    pub window: Option<WindowState>,

    #[cfg(feature = "editor")]
    pub graph_ui_state: Option<crate::editor::graph_ui::GraphUiState>,
}

impl PersistentState {
    pub fn build_state(&self, facade: &impl glium::backend::Facade) -> GraphState {
        let mut state = self.state.clone();
        state.apply_events_from_graph(&mut self.editor.graph, facade);
        state
    }

    pub fn new(
        editor: GraphEditorState,
        state: GraphState,
        window: Option<WindowState>,
        #[cfg(feature = "editor")] graph_ui_state: Option<crate::editor::graph_ui::GraphUiState>,
    ) -> Self {
        Self {
            editor,
            state,
            window,
            #[cfg(feature = "editor")]
            graph_ui_state,
        }
    }
}

#[derive(Default, Serialize, Deserialize, Clone)]
pub struct WindowState {
    pub res: (u32, u32),
    pub fullscreen: bool,
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
