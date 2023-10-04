use std::{collections::HashMap, time::Instant};

use egui_node_graph::NodeId;

use crate::common::animation::{DataUpdater, UpdateInfo};

use serde::{Deserialize, Serialize};

use super::graph_change_listener::{GraphChangeEvent, GraphUpdateListener, GraphUpdater};

#[derive(Default, Serialize, Deserialize, Clone)]
pub struct Animator {
    #[serde(with = "vectorize")]
    pub animations: HashMap<(NodeId, String), DataUpdater>,

    pub update_info: UpdateInfo,
}

impl GraphUpdateListener for Animator {
    fn graph_event(
        &mut self,
        graph: &mut super::def::Graph,
        facade: &impl glium::backend::Facade,
        event: super::graph_change_listener::GraphChangeEvent,
    ) {
        match self {
            GraphChangeEvent::DestroyedNode(node_id) => {
                self.animations.retain(|(id, _), _| id != *node_id);
            }
        }
    }
}

impl GraphUpdater for Animator {
    fn update(&mut self, graph: &mut super::def::Graph, facade: &impl glium::backend::Facade) {
        let elapsed_since_update = self.update_info.last_update.elapsed();
        let update_info = UpdateInfo::new(elapsed_since_update);

        for ((node_id, param_name), animation) in &self.animations {
            let maybe_input = graph.nodes[*node_id]
                .inputs
                .iter()
                .find(|(input_name, _)| input_name == param_name);

            if let Some((_, input_id)) = maybe_input {
                let input_id = *input_id;
                let input_param = &mut graph.inputs[input_id].value;
                animation.update_value(input_param, &update_info);
            }
        }

        self.update_info.last_update = Instant::now();
    }
}
