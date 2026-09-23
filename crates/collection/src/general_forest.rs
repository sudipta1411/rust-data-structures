use crate::{collection::Collection, general_tree::GeneralTree};

pub trait GeneralForest<T>: Collection {
    type Tree: GeneralTree<T>;

    fn trees(&self) -> &[Self::Tree];
    fn trees_mut(&mut self) -> &mut [Self::Tree];
    fn add_tree(&mut self, tree: Self::Tree) -> usize;
    fn add_root(&mut self, value: T) -> &mut Self::Tree;
    fn remove_tree(&mut self, index: usize) -> Option<Self::Tree>;
    fn append(&mut self, other: &mut Self);
    fn clear(&mut self);

    fn tree_count(&self) -> usize {
        self.trees().len()
    }
    fn get(&self, index: usize) -> Option<&Self::Tree> {
        self.trees().get(index)
    }
    fn get_mut(&mut self, index: usize) -> Option<&mut Self::Tree> {
        self.trees_mut().get_mut(index)
    }
    fn height(&self) -> usize {
        self.trees()
            .iter()
            .map(|tree| tree.height())
            .max()
            .unwrap_or(0)
    }
    fn contains<F>(&self, mut predicate: F) -> bool
    where
        F: FnMut(&T) -> bool,
    {
        for tree in self.trees() {
            if tree.contains(&mut predicate) {
                return true;
            }
        }
        false
    }

    fn preorder_traverse(&self) -> Vec<&T> {
        let mut values = Vec::new();
        for tree in self.trees() {
            values.extend(tree.preorder_traverse());
        }
        values
    }
    fn postorder_traverse(&self) -> Vec<&T> {
        let mut values = Vec::new();
        for tree in self.trees() {
            values.extend(tree.postorder_traverse());
        }
        values
    }
    fn level_order_traverse(&self) -> Vec<&T> {
        let mut values = Vec::new();
        for tree in self.trees() {
            values.extend(tree.level_order_traverse());
        }
        values
    }
}
