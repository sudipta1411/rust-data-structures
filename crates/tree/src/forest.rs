use collection::{
    collection::Collection, general_forest::GeneralForest, general_tree::GeneralTree,
};

use crate::tree::Tree;

#[derive(Debug)]
pub struct Forest<T> {
    trees: Vec<Tree<T>>,
}

impl<T> Forest<T> {
    pub fn new() -> Self {
        Self { trees: Vec::new() }
    }
}

impl<T> Collection for Forest<T> {
    fn len(&self) -> usize {
        self.trees.iter().map(|tree| tree.len()).sum()
    }
    fn is_empty(&self) -> bool {
        self.trees.iter().all(|tree| tree.is_empty())
    }
}

impl<T> GeneralForest<T> for Forest<T> {
    type Tree = Tree<T>;

    fn trees(&self) -> &[Self::Tree] {
        &self.trees
    }
    fn trees_mut(&mut self) -> &mut [Self::Tree] {
        &mut self.trees
    }
    fn add_tree(&mut self, tree: Self::Tree) -> usize {
        let index = self.trees.len();
        self.trees.push(tree);
        index
    }
    fn add_root(&mut self, value: T) -> &mut Self::Tree {
        self.trees.push(Tree::with_root(value));
        self.trees.last_mut().unwrap()
    }
    fn remove_tree(&mut self, index: usize) -> Option<Self::Tree> {
        if index >= self.trees.len() {
            return None;
        }
        Some(self.trees.remove(index))
    }
    fn append(&mut self, other: &mut Self) {
        self.trees.append(&mut other.trees);
    }
    fn clear(&mut self) {
        for tree in &mut self.trees {
            tree.clear();
        }
        self.trees.clear();
    }
}
