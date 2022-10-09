use std::{path::{Path, PathBuf}, fmt::Display, fs::read_dir};

use crate::{tree_view::Tree, isf::meta::{default_isf_path, IsfInfo}};




#[derive(Debug)]
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
    fn filter_item(&self, item: &TreeItem) -> bool {
        let text_pass = self.text.is_empty() || 
            item.name.to_lowercase().contains(&self.text.to_lowercase());

        let isf_pass = match item.isf {
            Some(ref isf) => {
                let has_inputs = isf.def.inputs.iter().any(|x| x.ty == isf::InputType::Image);

                (!self.image_inputs || has_inputs) && (!self.no_image_inputs || !has_inputs)
            },
            None => true,
        };

        text_pass && isf_pass && item.isf.is_some()
    }
}

pub struct TreeState{
    filter: FilterState,
    trees: Vec<Tree<TreeItem>>
}

impl Default for TreeState {
    fn default() -> Self {
        Self {
            trees: vec![default_isf_path().as_ref(), Path::new("C:\\ProgramData\\ISF")]
                .into_iter()
                .map(load_isf_tree)
                .collect(),

            filter: FilterState::default()
        }
    }
}

impl TreeState {
    pub fn draw(&mut self, ui: &mut egui::Ui) -> Option<&TreeItem> {
        let mut new_item = None;
        let mut search_changed = false;

        ui.heading("Node Types");

        search_changed |= ui.text_edit_singleline(&mut self.filter.text).changed();
        ui.horizontal(|ui| {
            ui.label("Image In");
            search_changed |= ui.toggle_value(&mut self.filter.image_inputs, "Some").clicked();
            search_changed |= ui.toggle_value(&mut self.filter.no_image_inputs, "None").clicked();
        });

        egui::ScrollArea::vertical().show(ui, |ui| {
            ui.set_min_width(128.0);

            for tree in &mut self.trees {
                if search_changed {
                    dbg!(&self.filter);
                    tree.map_mut(&mut |item| {
                        item.visible = self.filter.filter_item(item);
                    });
                }

                if let Some(selected_item) = tree.draw(ui, &|item| item.visible) {
                    new_item = Some(selected_item);
                    
                    
                    // self.add
                }
            }
        });

        new_item
    }
}

pub struct TreeItem {
    pub name: String,
    pub path: PathBuf,
    visible: bool,
    pub isf: Option<IsfInfo>
}

impl TreeItem {
    fn new(path: PathBuf) -> Self {
        let name = path.file_name().unwrap().to_str().unwrap().to_string();

        let isf = IsfInfo::new_from_path(&path).ok();

        Self {
            name,
            path,
            visible: true,
            isf
        }
    }
}

impl Display for TreeItem {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.name)
    }
}

fn load_isf_tree(path: &Path) -> Tree<TreeItem> {
    let info = TreeItem::new(path.clone().to_path_buf());

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