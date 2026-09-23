use std::cmp::Ordering;

use collection::collection::{Collection, SearchNode, SearchTree};

type Link<T> = Option<Box<Node<T>>>;

#[derive(Debug)]
pub struct Node<T> {
    value: T,
    left: Link<T>,
    right: Link<T>,
    height: usize,
}

impl<T> Node<T> {
    fn new(value: T) -> Self {
        Self {
            value,
            left: None,
            right: None,
            height: 1,
        }
    }

    fn link_height(link: &Link<T>) -> usize {
        link.as_ref().map_or(0, |node| node.height)
    }

    fn update_height(&mut self) {
        self.height = 1 + Self::link_height(&self.left).max(Self::link_height(&self.right));
    }

    fn balance_factor(&self) -> isize {
        Self::link_height(&self.left) as isize - Self::link_height(&self.right) as isize
    }
}

impl<T> SearchNode<T> for Node<T> {
    fn value(&self) -> &T {
        &self.value
    }
    fn left(&self) -> Option<&Self> {
        self.left.as_deref()
    }

    fn right(&self) -> Option<&Self> {
        self.right.as_deref()
    }
}

#[derive(Debug)]
pub struct AvlTree<T> {
    root: Link<T>,
    len: usize,
}

impl<T: Ord> AvlTree<T> {
    pub fn new() -> Self {
        Self { root: None, len: 0 }
    }

    fn rotate_right(mut root: Box<Node<T>>) -> Box<Node<T>> {
        let mut pivot = root.left.take().unwrap();
        root.left = pivot.right.take();
        root.update_height();
        pivot.right = Some(root);
        pivot.update_height();
        pivot
    }

    fn rotate_left(mut root: Box<Node<T>>) -> Box<Node<T>> {
        let mut pivot = root.right.take().unwrap();
        root.right = pivot.left.take();
        root.update_height();
        pivot.left = Some(root);
        pivot.update_height();
        pivot
    }

    fn rebalance(mut node: Box<Node<T>>) -> Box<Node<T>> {
        node.update_height();
        let balance = node.balance_factor();
        if balance > 1 {
            if node.left.as_ref().unwrap().balance_factor() < 0 {
                let left = node.left.take().unwrap();
                node.left = Some(Self::rotate_left(left));
            }
            return Self::rotate_right(node);
        }
        if balance < -1 {
            if node.right.as_ref().unwrap().balance_factor() > 0 {
                let right = node.right.take().unwrap();
                node.right = Some(Self::rotate_right(right));
            }
            return Self::rotate_left(node);
        }
        node
    }

    fn insert_into(link: &mut Link<T>, value: T) -> bool {
        let Some(node) = link.as_mut() else {
            *link = Some(Box::new(Node::new(value)));
            return true;
        };
        let inserted = match value.cmp(&node.value) {
            Ordering::Less => Self::insert_into(&mut node.left, value),
            Ordering::Greater => Self::insert_into(&mut node.right, value),
            Ordering::Equal => return false,
        };
        if inserted {
            let node = link.take().unwrap();
            *link = Some(Self::rebalance(node));
        }
        inserted
    }

    fn take_min(link: &mut Link<T>) -> T {
        let node = link.as_mut().unwrap();
        if node.left.is_none() {
            let mut node = link.take().unwrap();
            *link = node.right.take();
            return node.value;
        }
        let value = Self::take_min(&mut node.left);
        let node = link.take().unwrap();
        *link = Some(Self::rebalance(node));
        value
    }

    fn remove_from(link: &mut Link<T>, value: &T) -> bool {
        let Some(node) = link.as_mut() else {
            return false;
        };
        let removed = match value.cmp(&node.value) {
            Ordering::Less => Self::remove_from(&mut node.left, value),
            Ordering::Greater => Self::remove_from(&mut node.right, value),
            Ordering::Equal => {
                if node.left.is_none() {
                    *link = node.right.take();
                    return true;
                }
                if node.right.is_none() {
                    *link = node.left.take();
                    return true;
                }
                node.value = Self::take_min(&mut node.right);
                true
            }
        };
        if removed {
            let node = link.take().unwrap();
            *link = Some(Self::rebalance(node));
        }
        removed
    }
}

impl<T> Collection for AvlTree<T> {
    fn len(&self) -> usize {
        self.len
    }
}

impl<T: Ord> SearchTree<T> for AvlTree<T> {
    type Node = Node<T>;

    fn root(&self) -> Option<&Self::Node> {
        self.root.as_deref()
    }

    fn insert(&mut self, value: T) -> bool {
        let inserted = Self::insert_into(&mut self.root, value);
        if inserted {
            self.len += 1;
        }
        inserted
    }

    fn remove(&mut self, value: &T) -> bool {
        let removed = Self::remove_from(&mut self.root, value);
        if removed {
            self.len -= 1;
        }
        removed
    }

    fn height(&self) -> usize {
        Node::link_height(&self.root)
    }

    fn clear(&mut self) {
        let mut stack = Vec::new();
        if let Some(root) = self.root.take() {
            stack.push(root);
        }
        self.len = 0;
        while let Some(mut node) = stack.pop() {
            if let Some(left) = node.left.take() {
                stack.push(left);
            }
            if let Some(right) = node.right.take() {
                stack.push(right);
            }
        }
    }
}
