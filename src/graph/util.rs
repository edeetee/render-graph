use egui_node_graph::{NodeId, OutputId, InputId};

///Simplifies adding connections to nodes
pub trait GraphMutHelper<T>
where T: Copy,
    T: Into<&'static str>
{

    /// use connection name for [`Self::input_named()`]
    fn input(&mut self, node_id: NodeId, connection: T) -> InputId {
        self.input_named(node_id, connection, connection.into())
    }

    fn input_named(&mut self, node_id: NodeId, connection: T, name: &str) -> InputId;


    /// use connection name for [`Self::input_named()`]
    fn output(&mut self, node_id: NodeId, connection: T) -> OutputId{
        self.output_named(node_id, connection, connection.into())
    }

    fn output_named(&mut self, node_id: NodeId, connection: T, name: &str) -> OutputId;

    fn in_out(&mut self, node_id: NodeId, connection: T) -> (InputId, OutputId) {
        (self.input(node_id, connection), self.output(node_id, connection))
    }
}

// pub trait GraphConnectionHelper<T>{
//     fn get_node_from_output(&self, output: OutputId) -> Node<T>;
//     fn get_node_from_input(&self, input: InputId) -> Node<T>;
// }

// impl GraphConnectionHelper<NodeData> for Graph<NodeData, NodeConnectionTypes, NodeValueTypes>{
//     fn get_node_from_output(&self, output: OutputId) -> Node<NodeData> {
//         self[self[output].node]
//     }

//     fn get_node_from_input(&self, input: InputId) -> Node<NodeData> {
//         self[self[input].node]
//     }
// }