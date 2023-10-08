use egui_node_graph::{InputId, NodeDataTrait, NodeId, NodeResponse, OutputId, UserResponseTrait};
use glium::backend::Facade;

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
    pub fn vec_from_graph<N, C, V>(graph: &egui_node_graph::Graph<N, C, V>) -> Vec<Self> {
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

    pub fn from_response<UserResponse: UserResponseTrait, NodeData: NodeDataTrait>(
        response: &NodeResponse<UserResponse, NodeData>,
    ) -> Option<Self> {
        match response {
            NodeResponse::ConnectEventEnded { output, input } => {
                Some(GraphChangeEvent::Connected {
                    output_id: *output,
                    input_id: *input,
                })
            }
            NodeResponse::CreatedNode(node_id) => Some(GraphChangeEvent::CreatedNode(*node_id)),
            NodeResponse::DeleteNodeFull { node_id, node: _ } => {
                Some(GraphChangeEvent::DestroyedNode(*node_id))
            }
            NodeResponse::DisconnectEvent { output, input } => {
                Some(GraphChangeEvent::Disconnected {
                    output_id: *output,
                    input_id: *input,
                })
            }
            _ => None,
        }
    }
}

pub trait GraphUpdateListener<N, C, V> {
    #[must_use = "must handle the error possibility"]
    fn graph_event(
        &mut self,
        graph: &mut egui_node_graph::Graph<N, C, V>,
        facade: &impl Facade,
        event: GraphChangeEvent,
    ) -> anyhow::Result<()>;
}

pub trait MultipleUpdatesListener<N, C, V>: GraphUpdateListener<N, C, V> {
    #[must_use = "must handle the error possibility"]
    fn apply_events_from_graph(
        &mut self,
        graph: &mut egui_node_graph::Graph<N, C, V>,
        facade: &impl Facade,
    ) -> anyhow::Result<()>;

    fn new_from_graph(
        graph: &mut egui_node_graph::Graph<N, C, V>,
        facade: &impl Facade,
    ) -> anyhow::Result<Self>
    where
        Self: Default,
    {
        let mut new = Self::default();
        new.apply_events_from_graph(graph, facade)?;
        Ok(new)
    }
}

impl<T: GraphUpdateListener<N, C, V>, N, C, V> MultipleUpdatesListener<N, C, V> for T {
    fn apply_events_from_graph(
        &mut self,
        graph: &mut egui_node_graph::Graph<N, C, V>,
        facade: &impl Facade,
    ) -> anyhow::Result<()> {
        for event in GraphChangeEvent::vec_from_graph(graph) {
            self.graph_event(graph, facade, event)?;
        }
        Ok(())
    }
}

pub trait GraphUpdater<N, C, V> {
    #[must_use = "must handle the error possibility"]
    fn update(
        &mut self,
        graph: &mut egui_node_graph::Graph<N, C, V>,
        facade: &impl Facade,
    ) -> anyhow::Result<()>;
}
