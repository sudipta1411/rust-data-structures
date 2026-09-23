use std::collections::VecDeque;

use crate::{
    collection::{Collection, TraversalType},
    tree_node::TreeNode,
};

pub trait GeneralTree<T>: Collection {
    type Node: TreeNode<T>;

    fn root(&self) -> Option<&Self::Node>;
    fn root_mut(&mut self) -> Option<&mut Self::Node>;
    fn replace_root(&mut self, root: Self::Node) -> Option<Self::Node>;
    fn take_root(&mut self) -> Option<Self::Node>;
    fn clear(&mut self);
    fn height(&self) -> usize {
        let mut height = 0;
        let mut stack = Vec::new();
        if let Some(root) = self.root() {
            stack.push((root, 1));
        }
        while let Some((node, depth)) = stack.pop() {
            height = height.max(depth);
            for child in node.children() {
                stack.push((child, depth + 1));
            }
        }
        height
    }
    fn contains<F>(&self, mut predicate: F) -> bool
    where
        F: FnMut(&T) -> bool,
    {
        let mut stack = Vec::new();
        if let Some(root) = self.root() {
            stack.push(root);
        }
        while let Some(node) = stack.pop() {
            if predicate(node.value()) {
                return true;
            }
            stack.extend(node.children().iter().rev());
        }
        false
    }
    fn preorder_traverse(&self) -> Vec<&T> {
        let mut values = Vec::new();
        let mut stack = Vec::new();
        if let Some(root) = self.root() {
            stack.push(root);
        }
        while let Some(node) = stack.pop() {
            values.push(node.value());
            stack.extend(node.children().iter().rev());
        }
        values
    }
    fn postorder_traverse(&self) -> Vec<&T> {
        let mut values = Vec::new();
        let mut stack = Vec::new();
        if let Some(root) = self.root() {
            stack.push((root, false));
        }
        while let Some((node, visited)) = stack.pop() {
            if visited {
                values.push(node.value());
                continue;
            }
            stack.push((node, true));
            for child in node.children().iter().rev() {
                stack.push((child, false));
            }
        }
        values
    }
    fn level_order_traverse(&self) -> Vec<&T> {
        let mut values = Vec::new();
        let mut queue = VecDeque::new();
        if let Some(root) = self.root() {
            queue.push_back(root);
        }
        while let Some(node) = queue.pop_front() {
            values.push(node.value());
            queue.extend(node.children());
        }
        values
    }
    fn traverse(&self, traversal_type: TraversalType) -> Option<Vec<&T>> {
        match traversal_type {
            TraversalType::InOrder => None,
            TraversalType::LevelOrder => Some(self.level_order_traverse()),
            TraversalType::PostOrder => Some(self.postorder_traverse()),
            TraversalType::PreOrder => Some(self.preorder_traverse()),
        }
    }
}
