pub trait TreeNode<T>: Sized {
    fn value(&self) -> &T;
    fn value_mut(&mut self) -> &mut T;
    fn children(&self) -> &[Self];
    fn children_mut(&mut self) -> &mut [Self];
    fn add_child(&mut self, value: T) -> &mut Self;
    fn add_subtree(&mut self, subtree: Self) -> &mut Self;
    fn remove_child(&mut self, index: usize) -> Option<Self>;
    fn child_count(&self) -> usize {
        self.children().len()
    }
    fn is_leaf(&self) -> bool {
        self.children().is_empty()
    }
}
