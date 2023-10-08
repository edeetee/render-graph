use egui_node_graph::{InputParam, NodeId};

use slotmap::SecondaryMap;

pub type InputParams<'a, C, V> = Vec<(&'a str, &'a InputParam<C, V>)>;
pub type ProcessedInputs<'a, C, V, OUT> = Vec<(&'a str, &'a InputParam<C, V>, Option<OUT>)>;

///Trait for internal mapping over a graph
pub trait GraphMap<C, V, FOnNode, OUT: Clone>
where
    FOnNode: FnMut(NodeId, ProcessedInputs<'_, C, V, OUT>) -> Option<OUT>,
{
    fn map_with_inputs(
        &self,
        node_id: NodeId,
        f_on_node: &mut FOnNode,
        cache: &mut SecondaryMap<NodeId, Option<OUT>>,
    ) -> Option<OUT>;

    fn compute_inputs(
        &self,
        node_id: NodeId,
        f_on_node: &mut FOnNode,
        cache: &mut SecondaryMap<NodeId, Option<OUT>>,
    ) -> ProcessedInputs<C, V, OUT>;
}

impl<N, C, V, FOnNode, OUT: Clone> GraphMap<C, V, FOnNode, OUT> for egui_node_graph::Graph<N, C, V>
where
    FOnNode: FnMut(NodeId, ProcessedInputs<'_, C, V, OUT>) -> Option<OUT>,
{
    ///Call f for each node in correct order, ending on node_id\
    ///
    /// # Type arguments
    /// OUT: type that may come out of a
    fn map_with_inputs(
        &self,
        node_id: NodeId,
        f_on_node: &mut FOnNode,
        cache: &mut SecondaryMap<NodeId, Option<OUT>>,
    ) -> Option<OUT> {
        let computed_inputs = self.compute_inputs(node_id, f_on_node, cache);
        let result = f_on_node(node_id, computed_inputs);
        result
    }

    fn compute_inputs(
        &self,
        node_id: NodeId,
        f_on_node: &mut FOnNode,
        cache: &mut SecondaryMap<NodeId, Option<OUT>>,
    ) -> ProcessedInputs<C, V, OUT> {
        self[node_id]
            .inputs
            .iter()
            .map(|(name, input_id)| {
                //if input is connected, generate the value

                let results = self
                    .connection(*input_id)
                    .map(|output_id| {
                        //we get to process a node!
                        let processing_node_id = self[output_id].node;

                        //add input to cache if doesn't exist
                        if !cache.contains_key(processing_node_id) {
                            let value = self.map_with_inputs(processing_node_id, f_on_node, cache);
                            cache.insert(processing_node_id, value);
                        }

                        cache[processing_node_id].clone()
                    })
                    .flatten();

                (name.as_str(), &self[*input_id], results)
            })
            .collect()
    }
}
