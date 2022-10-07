use std::fmt::Display;
use egui::Ui;

#[derive(Hash)]
pub enum Tree<T: Display> {
    Leaf(T),
    Branch(T, Vec<Tree<T>>),
}

impl<T: Display> Tree<T> {
    pub fn draw(&self, ui: &mut Ui) -> Option<&T> {
        // let all_kinds = NodeTypes::get_all();
        match self {
            Tree::Leaf(leaf) => {
                if ui.button(leaf.to_string()).clicked() {
                    Some(&leaf)
                } else {
                    None
                }
            },
            Tree::Branch(name, branch) => {
                ui.collapsing(name.to_string(), |ui| {
                    let mut selected = None;

                    for tree in branch {
                        if let Some(selected_item) = tree.draw(ui){
                            selected = Some(selected_item);
                        }
                    }

                    selected
                }).body_returned.flatten()
            }
        }
    }
}