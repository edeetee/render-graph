use std::fmt::Display;
use egui::{Ui, Widget};

///Linked struct for displaying a tree of node templates
#[derive(Hash)]
pub enum Tree<Leaf, Branch> {
    Leaf(Leaf),
    Branch(Branch, Vec<Tree<Leaf, Branch>>),
}

pub trait RefWidget {
    fn ui(&self, ui: &mut Ui) -> egui::Response;
}

//implementation for copyable items
impl<Leaf: Copy, Branch: Copy> Tree<Leaf, Branch> where {
    ///Mutably iterate over the tree
    pub fn map_leaf(&self, f: &mut impl FnMut(Leaf)) {
        match self {
            Tree::Leaf(item) => f(*item),
            Tree::Branch(_item, children) => {
                // f(item);
                for child in children {
                    child.map_leaf(f);
                }
            }
        }
    }

    // pub fn map<R, BR, DrawLeafFn: Fn(&mut Ui, &mut Leaf) -> R, DrawBranchFn: FnMut(&mut Ui, &DrawLeafFn) -> BR>
    // (&mut self, ui: &mut Ui, open_state: Option<bool>, filter: &impl Fn(&Leaf) -> bool, drawLeaf: &DrawLeafFn, drawBranch: &DrawBranchFn) -> BR {
    //     match self {
    //         Tree::Leaf(leaf) => {
    //             vec![drawLeaf(ui, leaf)]
    //         },

    //         Tree::Branch(name, branch) => {
    //             drawBranch(ui, drawLeaf)
    //         }
    //     }
    // }

    ///draw all elements of the tree with a filter. Returns a clicked leaf
    pub fn draw<'a, R>(
        &'a mut self, 
        ui: &mut Ui, 
        open_state: Option<bool>, 
        draw: &mut impl FnMut(&mut Ui, Leaf) -> R,
        branch_header: &mut impl Fn(Branch) -> String,
    ) -> Vec<R> {
        match self {
            Tree::Leaf(leaf) => {
                vec![draw(ui, *leaf)]
            },
            Tree::Branch(branch, children) => {
                egui::CollapsingHeader::new(branch_header(*branch))
                    .open(open_state)
                    .show(ui, |ui| {
                        ui.with_layout(egui::Layout::left_to_right(egui::Align::Min).with_main_wrap(true), |ui|{
                            children.iter_mut().flat_map(|child| child.draw(ui, open_state, draw, branch_header)).collect()
                        }).inner
                    })
                    .body_returned.unwrap_or_default()
            }
        }
    }
}