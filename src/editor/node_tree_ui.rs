use std::{path::{Path, PathBuf}, fmt::Display, fs::read_dir};

use serde::Serialize;

use crate::{tree_view::Tree, isf::meta::{default_isf_path, IsfInfo}, common::connections::ConnectionType};

use crate::graph::{node_types::NodeType, };


#[derive(Debug, Serialize)]
struct FilterState{
    image_inputs: bool,
    no_image_inputs: bool,
    text: String,
}

impl Default for FilterState {
    fn default() -> Self {
        Self {
            image_inputs: false,
            no_image_inputs: false,
            text: String::new(),
        }
    }
}

impl FilterState {
    fn filter_item(&self, item: &LeafItem) -> bool {
        let text_pass = self.text.is_empty() || 
            item.ty.get_name().to_lowercase().contains(&self.text.to_lowercase());

        let image_input_pass = {
            let has_inputs = item.ty.get_input_types().iter().any(|x| x.ty == ConnectionType::Texture2D);

            (!self.image_inputs || has_inputs) && (!self.no_image_inputs || !has_inputs)
        };

        text_pass && image_input_pass
    }
}

///Holds the data for the tree vi
pub struct TreeState{
    filter: FilterState,
    trees: Vec<Tree<LeafItem, BranchItem>>
}

impl Default for TreeState {
    fn default() -> Self {
        let default_nodes = NodeType::get_builtin()
            .into_iter()
            .map(LeafItem::new)
            .map(Tree::Leaf);

        let cargo_shaders = Path::new(env!("CARGO_MANIFEST_DIR")).join("shaders");
        let default_shaders = default_isf_path();

        let isf_nodes = vec![cargo_shaders.as_ref(), default_shaders.as_ref()]
            .into_iter()
            .filter_map(load_isf_tree);

        Self {
            trees: default_nodes.chain(isf_nodes).collect(),
            filter: FilterState::default()
        }
    }
}

impl TreeState {
    pub fn draw(&mut self, ui: &mut egui::Ui) -> Option<&LeafItem> {
        let mut new_item = None;
        let mut search_changed = false;

        ui.heading("Node Types");

        search_changed |= ui.text_edit_singleline(&mut self.filter.text).changed();
        ui.horizontal(|ui| {
            ui.label("Image In");
            search_changed |= ui.toggle_value(&mut self.filter.image_inputs, "Some").clicked();
            search_changed |= ui.toggle_value(&mut self.filter.no_image_inputs, "None").clicked();
        });

        let open_state = if !search_changed{
            None
        } else if self.filter.text.is_empty() {
            None
        } else {
            Some(true)
        };

        egui::ScrollArea::vertical().show(ui, |ui| {
            ui.set_min_width(128.0);

            for tree in &mut self.trees {
                if search_changed {
                    // dbg!(&self.filter);
                    tree.map_mut(&mut |item| {
                        item.visible = self.filter.filter_item(item);
                    });
                }

                if let Some(selected_item) = tree.draw(ui, open_state, &|item| item.visible) {
                    new_item = Some(selected_item);
                }
            }
        });

        new_item
    }
}

pub struct LeafItem {
    visible: bool,
    pub ty: NodeType
}

impl LeafItem {
    fn new(ty: NodeType) -> Self {
        Self {
            visible: true,
            ty
        }
    }

    fn new_from_isf(isf_path: PathBuf) -> Option<Self> {
        let info = IsfInfo::new_from_path(&isf_path).ok()?;

        Some(Self::new(NodeType::Isf { info }))
    }
}

impl Display for LeafItem {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.ty)
    }
}

pub struct BranchItem {
    pub name: String,
}

impl Display for BranchItem {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.name)
    }
}

fn load_isf_tree(path: &Path) -> Option<Tree<LeafItem, BranchItem>> {
    if path.is_dir() {
        let branch_inner = read_dir(path)
            .unwrap()
            .into_iter()
            .filter_map(|dir| {
                load_isf_tree(&dir.unwrap().path())
            })
            .collect();

        let info = BranchItem {
            name: path.file_name().unwrap().to_str().unwrap().to_string(),
        };

        Some(Tree::Branch(info, branch_inner))
    } else {
        let info = LeafItem::new_from_isf(path.clone().to_path_buf())?;
        Some(Tree::Leaf(info))
    }
}