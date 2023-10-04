use egui_node_graph::{InputId, NodeId, OutputId};
use glium::backend::Facade;

use super::def::Graph;

#[derive(Clone, Copy)]
pub enum GraphChangeEvent {
    CreatedNode(NodeId),
    DestroyedNode(NodeId),

    Connected {
        output_id: OutputId,
        input_id: InputId,
    },
    Disconnected {
        output_id: OutputId,
        input_id: InputId,
    },
}

impl GraphChangeEvent {
    #[must_use = "Use the vec of node responses to load callbacks"]
    pub fn vec_from_graph(graph: &Graph) -> Vec<Self> {
        let new_nodes = graph
            .nodes
            .iter()
            .map(|(node_id, ..)| GraphChangeEvent::CreatedNode(node_id));

        let new_connections =
            graph
                .connections
                .iter()
                .map(|(input, output)| GraphChangeEvent::Connected {
                    output_id: *output,
                    input_id: input,
                });

        new_nodes.chain(new_connections).collect()
    }
}

pub trait GraphUpdateListener {
    fn graph_event(&mut self, graph: &mut Graph, facade: &impl Facade, event: GraphChangeEvent);
}

pub trait MultipleUpdatesListener: GraphUpdateListener {
    fn apply_events_from_graph(&mut self, graph: &mut Graph, facade: &impl Facade);
}

impl<T: GraphUpdateListener> MultipleUpdatesListener for T {
    fn apply_events_from_graph(&mut self, graph: &mut Graph, facade: &impl Facade) {
        for event in GraphChangeEvent::vec_from_graph(graph) {
            self.graph_event(graph, facade, event);
        }
    }
}

pub trait GraphUpdater {
    fn update(&mut self, graph: &mut Graph, facade: &impl Facade);
}
