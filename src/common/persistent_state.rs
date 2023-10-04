use crate::{
    graph::{
        def::{GraphEditorState, GraphState},
        MultipleUpdatesListener,
    },
    util::{read_from_json_file, write_to_json_file},
};
use serde::{Deserialize, Serialize};
use std::{env, path::PathBuf};

#[derive(Default, Serialize, Deserialize)]
pub struct PersistentState(HydratedPersistentState);

#[derive(Default, Serialize, Deserialize)]
#[serde(default)]
pub struct HydratedPersistentState {
    pub editor: GraphEditorState,
    pub state: GraphState,
    pub window: Option<WindowState>,

    #[cfg(feature = "editor")]
    pub graph_ui_state: Option<crate::editor::graph_ui::GraphUiState>,
}

impl PersistentState {
    pub fn new(state: HydratedPersistentState) -> Self {
        Self(state)
    }

    pub fn window_state(&self) -> Option<WindowState> {
        self.0.window.clone()
    }

    pub fn hydrate(mut self, facade: &impl glium::backend::Facade) -> HydratedPersistentState {
        let mut state = self.0.state;
        state.apply_events_from_graph(&mut self.0.editor.graph, facade);
        HydratedPersistentState { state, ..self.0 }
    }

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
