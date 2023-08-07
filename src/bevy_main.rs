use bevy::prelude::*;
use bevy_egui::{egui, EguiContexts, EguiPlugin};

use crate::{editor::graph_ui::GraphUi, common::persistent_state::PersistentState};

pub fn main() {

    let state = PersistentState::load_from_default_path();
    let mut graph_ui = GraphUi::new_from_persistent(state, &display, &mut egui_glium);

    App::new()
        // .add_plugins(DefaultPlugins)
        .add_plugins(EguiPlugin)
        // Systems that create Egui widgets should be run during the `CoreSet::Update` set,
        // or after the `EguiSet::BeginFrame` system (which belongs to the `CoreSet::PreUpdate` set).
        .add_systems(Update, )
        .run();
}