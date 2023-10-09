slotmap::new_key_type! {
    pub struct LeafIndex;
}

slotmap::new_key_type! {
    pub struct BranchIndex;
}

use slotmap::SlotMap;

///Linked struct for displaying a tree of node templates
#[derive(Hash, Clone)]
pub enum TreeStructure {
    Leaf(LeafIndex),
    Branch(BranchIndex, Vec<TreeStructure>),
}

pub struct Tree<BranchItem, LeafItem> {
    pub branches: SlotMap<BranchIndex, BranchItem>,
    pub leaves: SlotMap<LeafIndex, LeafItem>,
    pub tree: Vec<TreeStructure>,
}

// impl<B, L> Tree<B, L> {
//     pub fn new<
//         F: Fn(I, &mut SlotMap<BranchIndex, B>, &mut SlotMap<LeafIndex, L>) -> Option<TreeStructure>,
//         I,
//     >(
//         builder: &F,
//         input: I,
//     ) -> Option<Self> {
//         let mut branches = SlotMap::default();
//         let mut leaves = SlotMap::default();

//         // let tree = builder(input, &mut branches, &mut leaves)?;

//         Some(Self {
//             branches,
//             leaves,
//             tree,
//         })
//     }

//     pub fn extend<
//         F: Fn(I, &mut SlotMap<BranchIndex, B>, &mut SlotMap<LeafIndex, L>) -> Option<TreeStructure>,
//         I,
//     >(
//         &mut self,
//         builder: &F,
//         input: I,
//         new_top: B,
//     ) {
//         if let Some(tree) = builder(input, &mut self.branches, &mut self.leaves) {
//             let new_top = self.branches.insert(new_top);
//             let old_top = std::mem::replace(&mut self.tree, TreeStructure::Branch(new_top, vec![]));
//             self.tree = TreeStructure::Branch(new_top, vec![old_top, tree]);
//         }
//     }
// }

impl TreeStructure {
    ///Mutably iterate over the tree
    pub fn map_leaf(&self, f: &mut impl FnMut(LeafIndex)) {
        match self {
            TreeStructure::Leaf(item) => f(*item),
            TreeStructure::Branch(_item, children) => {
                // f(item);
                for child in children {
                    child.map_leaf(f);
                }
            }
        }
    }
}
