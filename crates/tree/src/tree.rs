use collection::{collection::Collection, general_tree::GeneralTree, tree_node::TreeNode};

#[derive(Debug)]
pub struct Node<T> {
    value: T,
    children: Vec<Node<T>>,
}

#[derive(Debug)]
pub struct Tree<T> {
    root: Option<Node<T>>,
}

impl<T> Node<T> {
    pub fn new(value: T) -> Self {
        Self {
            value,
            children: Vec::new(),
        }
    }
}

impl<T> Tree<T> {
    pub fn new() -> Self {
        Self { root: None }
    }
    pub fn with_root(value: T) -> Self {
        Self {
            root: Some(Node::new(value)),
        }
    }
}

impl<T> TreeNode<T> for Node<T> {
    fn value(&self) -> &T {
        &self.value
    }
    fn value_mut(&mut self) -> &mut T {
        &mut self.value
    }
    fn children(&self) -> &[Self] {
        &self.children
    }
    fn children_mut(&mut self) -> &mut [Self] {
        &mut self.children
    }
    fn add_child(&mut self, value: T) -> &mut Self {
        self.add_subtree(Node::new(value))
    }
    fn add_subtree(&mut self, subtree: Self) -> &mut Self {
        self.children.push(subtree);
        self.children.last_mut().unwrap()
    }
    fn remove_child(&mut self, index: usize) -> Option<Self> {
        if index >= self.children.len() {
            return None;
        }
        Some(self.children.remove(index))
    }
}

impl<T> Collection for Tree<T> {
    fn len(&self) -> usize {
        let mut count = 0;
        let mut stack = Vec::new();
        if let Some(root) = self.root() {
            stack.push(root);
        }
        while let Some(node) = stack.pop() {
            count += 1;
            stack.extend(node.children());
        }
        count
    }
    fn is_empty(&self) -> bool {
        self.root().is_none()
    }
}

impl<T> GeneralTree<T> for Tree<T> {
    type Node = Node<T>;

    fn root(&self) -> Option<&Self::Node> {
        self.root.as_ref()
    }
    fn root_mut(&mut self) -> Option<&mut Self::Node> {
        self.root.as_mut()
    }
    fn replace_root(&mut self, root: Self::Node) -> Option<Self::Node> {
        self.root.replace(root)
    }
    fn take_root(&mut self) -> Option<Self::Node> {
        self.root.take()
    }
    fn clear(&mut self) {
        let mut stack = Vec::new();
        if let Some(root) = self.root.take() {
            stack.push(root);
        }
        while let Some(mut node) = stack.pop() {
            stack.append(&mut node.children);
        }
    }
}
