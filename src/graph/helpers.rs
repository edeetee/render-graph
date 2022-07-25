use egui_node_graph::{NodeId, OutputId, InputId};
pub trait GraphHelper<T>
where T: Copy,
    T: Into<&'static str>
{
    fn input(&mut self, node_id: NodeId, connection: T) -> InputId {
        self.input_named(node_id, connection, connection.into())
    }

    fn input_named(&mut self, node_id: NodeId, connection: T, name: &str) -> InputId;

    fn output(&mut self, node_id: NodeId, connection: T) -> OutputId{
        self.output_named(node_id, connection, connection.into())
    }

    fn output_named(&mut self, node_id: NodeId, connection: T, name: &str) -> OutputId;

    fn in_out(&mut self, node_id: NodeId, connection: T) -> (InputId, OutputId) {
        (self.input(node_id, connection), self.output(node_id, connection))
    }
}