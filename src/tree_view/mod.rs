use std::fmt::Display;
use egui::Ui;

///Linked struct for displaying a tree of node templates
#[derive(Hash)]
pub enum Tree<Leaf, Branch> {
    Leaf(Leaf),
    Branch(Branch, Vec<Tree<Leaf, Branch>>),
}

// #[derive(PartialEq, Eq)]
// enum TreeUiEvent {
//     ExpandAll
// }

impl<Leaf: Display, Branch: Display> Tree<Leaf, Branch> {

    ///Mutably iterate over the tree
    pub fn map_mut(&mut self, f: &mut impl FnMut(&mut Leaf)) {
        match self {
            Tree::Leaf(item) => f(item),
            Tree::Branch(_item, children) => {
                // f(item);
                for child in children {
                    child.map_mut(f);
                }
            }
        }
    }

    ///draw all elements of the tree with a filter. Returns a clicked leaf
    pub fn draw(&self, ui: &mut Ui, open_state: Option<bool>, filter: &impl Fn(&Leaf) -> bool) -> Option<&Leaf> {
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
                egui::CollapsingHeader::new(name.to_string())
                    // .default_open(true)
                    .open(open_state)
                    .show(ui, |ui| {
                        let mut selected = None;

                        for tree in branch {
                            if let Some(selected_item) = tree.draw(ui, open_state, filter){
                                selected = Some(selected_item);
                            }
                        }
    
                        selected
                    })
                    .body_returned.flatten()
            }
        }
    }
}