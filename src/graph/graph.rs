use std::{ops::{Index, IndexMut}, fs::read_dir, path::{Path, PathBuf}, fmt::Display};


use egui_node_graph::{GraphEditorState, NodeId, Node, InputParam, Graph, NodeTemplateTrait};
use slotmap::SecondaryMap;

use crate::{isf::meta::{default_isf_path, try_read_isf}, tree_view::Tree};

use super::{def::{GraphState, NodeData, GraphResponse, NodeConnectionTypes, NodeValueTypes, EditorState}, node_types::{AllNodeTypes, NodeTypes}};

// #[derive(Default)]
pub struct ShaderGraph(pub(super) EditorState);

impl Default for ShaderGraph {
    fn default() -> Self {
        Self(GraphEditorState::new(1.0, GraphState::default()))
    }
}

impl Index<NodeId> for ShaderGraph {
    type Output = Node<NodeData>;

    fn index(&self, index: NodeId) -> &Self::Output {
        &self.0.graph[index]
    }
}

impl IndexMut<NodeId> for ShaderGraph {
    fn index_mut(&mut self, index: NodeId) -> &mut Self::Output {
        &mut self.0.graph[index]
    }
}

impl ShaderGraph {
    pub fn graph_ref(&self) -> &Graph<NodeData, NodeConnectionTypes, NodeValueTypes> {
        &self.0.graph
    }

    ///Call f for each node in correct order, ending on node_id\
    /// 
    /// # Type arguments
    /// OUT: type that may come out of a 
    pub fn map_with_inputs<FOnNode, OUT: Clone>(&self, node_id: NodeId, f_on_node: &mut FOnNode, cache: &mut SecondaryMap<NodeId, OUT>) -> OUT 
        where FOnNode: FnMut(NodeId, Vec<(&str, &InputParam<NodeConnectionTypes, NodeValueTypes>, Option<OUT>)>) -> OUT
    {
        let computed_inputs = self.0.graph[node_id].inputs.iter()
            .map(|(name, input_id)| {
                //if input is connected, generate the value

                let process_input = self.0.graph.connection(*input_id).map(|output_id| {
                    //we get to process a node!
                    let input_node_id = self.0.graph[output_id].node;

                    //add input to cache if doesn't exist
                    if !cache.contains_key(input_node_id){
                        let value = self.map_with_inputs(input_node_id, f_on_node, cache);
                        cache.insert(input_node_id, value);
                    }

                    cache[input_node_id].clone()
                });

                let input_param = self.0.graph.get_input(*input_id);

                (name.as_str(), input_param, process_input)
            })
            .collect();

        let result = f_on_node(node_id, computed_inputs);

        result
    }

    pub fn add_node(&mut self, node_kind: NodeTypes, position: egui::Pos2) -> NodeId {
        let new_node = self.0.graph.add_node(
            node_kind.node_graph_label(),
            node_kind.user_data(),
            |graph, node_id| node_kind.build_node(graph, node_id),
        );
        self.0.node_positions.insert(
            new_node,
            position,
        );
        self.0.node_order.push(new_node);

        new_node
    }

    pub fn draw(&mut self, ctx: &egui::Context) -> egui_node_graph::GraphResponse<GraphResponse, NodeData> {

        let local_tree = load_isf_tree(&default_isf_path());
        let standard_tree = load_isf_tree(Path::new("C:\\ProgramData\\ISF"));

        let mut new_node_ty = None;

        egui::SidePanel::left("tree_view").show(ctx, |ui| {
            ui.heading("Node Types");
            egui::ScrollArea::vertical().show(ui, |ui| {
                ui.set_min_width(256.0);
                for tree in vec![local_tree, standard_tree] {
                    if let Some(selected_item) = tree.draw(ui) {
                        // dbg!(selected_item);
                        match try_read_isf(selected_item.path.clone()){
                            Ok((path_info, isf)) => {
                                new_node_ty = Some(NodeTypes::Isf { file: path_info, isf });
                                
                            },
                            Err(e) => {
                                println!("{e}");
                            }
                        }
                        // self.add
                    }
                }
            });
        });

        egui::CentralPanel::default().show(ctx, |ui| {
            egui::widgets::global_dark_light_mode_switch(ui);
            let mut responses = vec![];

            let editor_rect = ui.max_rect();

            if let Some(node_ty) = new_node_ty {
                let pos = editor_rect.left_top() - self.0.pan_zoom.pan;
                let new_node_id = self.add_node(node_ty, pos);
                responses.push(egui_node_graph::NodeResponse::CreatedNode(new_node_id));
            }

            if ui.ui_contains_pointer() {
                self.0.pan_zoom.pan += ctx.input().scroll_delta;
                self.0.pan_zoom.zoom *= ctx.input().zoom_delta();
            }

            let mut graph_resp = self.0.draw_graph_editor(ui, AllNodeTypes);

            graph_resp.node_responses.append(&mut responses);
            
            // responses.append(&mut );

            graph_resp
        }).inner
    }
}


struct TreePath {
    name: String,
    path: PathBuf
}

impl TreePath {
    fn new(path: PathBuf) -> Self {
        let name = path.file_name().unwrap().to_str().unwrap().to_string();

        Self {
            name,
            path
        }
    }
}

impl Display for TreePath {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.name)
    }
}

fn load_isf_tree(path: &Path) -> Tree<TreePath> {
    let info = TreePath::new(path.clone().to_path_buf());

    if path.is_dir() {
        let branch_inner = read_dir(path)
            .unwrap()
            .into_iter()
            .map(|dir| {
                load_isf_tree(&dir.unwrap().path())
            })
            .collect();

        Tree::Branch(info, branch_inner)
    } else {
        Tree::Leaf(info)
    }
}