use std::fmt::Display;
use egui::Ui;

#[derive(Hash)]
pub enum Tree<T> {
    Leaf(T),
    Branch(T, Vec<Tree<T>>),
}

impl<T: Display> Tree<T> {
    pub fn map_mut(&mut self, f: &mut impl FnMut(&mut T)) {
        match self {
            Tree::Leaf(item) => f(item),
            Tree::Branch(item, children) => {
                f(item);
                for child in children {
                    child.map_mut(f);
                }
            }
        }
    }

    pub fn draw(&self, ui: &mut Ui, filter: &impl Fn(&T) -> bool) -> Option<&T> {
        // let all_kinds = NodeTypes::get_all();
        match self {
            Tree::Leaf(leaf) => {
                if filter(leaf) {
                    if ui.button(leaf.to_string()).clicked() {
                        Some(&leaf)
                    } else {
                        None
                    }
                } else {
                    None
                }
            },
            Tree::Branch(name, branch) => {
                ui.collapsing(name.to_string(), |ui| {
                    let mut selected = None;

                    for tree in branch {
                        if let Some(selected_item) = tree.draw(ui, filter){
                            selected = Some(selected_item);
                        }
                    }

                    selected
                }).body_returned.flatten()
            }
        }
    }
}