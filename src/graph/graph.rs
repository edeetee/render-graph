use std::{path::PathBuf};



use egui_node_graph::{NodeId, InputParam};


use slotmap::{SecondaryMap};
use crate::{common::{def::{UiValue}, connections::ConnectionType}, util::read_from_json_file, graph::{def::NodeResponse}};

use super::{def::{Graph, GraphEditorState}, GraphChangeEvent};

pub type InputParams<'a> = Vec<(&'a str, &'a InputParam<ConnectionType, UiValue>)>;
pub type ProcessedInputs<'a, OUT> = Vec<(&'a str, &'a InputParam<ConnectionType, UiValue>, Option<OUT>)>;

pub trait GraphUtils {
    fn map_with_inputs<FOnNode, OUT: Clone>(&self, node_id: NodeId, f_on_node: &mut FOnNode, cache: &mut SecondaryMap<NodeId, Option<OUT>>) -> Option<OUT> 
        where FOnNode: FnMut(NodeId, ProcessedInputs<'_, OUT>) -> Option<OUT>;
}

impl GraphUtils for Graph {
    ///Call f for each node in correct order, ending on node_id\
    /// 
    /// # Type arguments
    /// OUT: type that may come out of a 
    fn map_with_inputs<FOnNode, OUT: Clone>(&self, node_id: NodeId, f_on_node: &mut FOnNode, cache: &mut SecondaryMap<NodeId, Option<OUT>>) -> Option<OUT> 
        where FOnNode: FnMut(NodeId, ProcessedInputs<'_, OUT>) -> Option<OUT>
    {
        let computed_inputs = self[node_id].inputs.iter()
            .map(|(name, input_id)| {
                //if input is connected, generate the value

                let process_input = self.connection(*input_id).map(|output_id| {
                    //we get to process a node!
                    let input_node_id = self[output_id].node;

                    //add input to cache if doesn't exist
                    if !cache.contains_key(input_node_id){
                        let value = self.map_with_inputs(input_node_id, f_on_node, cache);
                        cache.insert(input_node_id, value);
                    }

                    cache[input_node_id].clone()
                }).flatten();

                let input_param = &self[*input_id];

                (name.as_str(), input_param, process_input)
            })
            .collect();

        let result = f_on_node(node_id, computed_inputs);

        result
    }
}



#[must_use="Use the vec of node responses to load callbacks"]
pub fn load_from_file_or_default(file: &PathBuf) -> GraphEditorState {
    match read_from_json_file::<GraphEditorState>(file) {
        Ok(graph_state) => {
            println!("Loaded save file from {file:?}");
            graph_state
        }
        Err(err) => {
            eprintln!("Failed to read default save {file:?} ({err:?}). Using new graph");
            GraphEditorState::default()
        },
    }
}
