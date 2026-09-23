use collection::collection::{Collection, SearchNode, SearchTree};

type Link<T> = Option<Box<Node<T>>>;

#[derive(Debug)]
pub struct Node<T> {
    value: T,
    left: Link<T>,
    right: Link<T>,
}

#[derive(Debug)]
pub struct BinarySearchTree<T> {
    root: Link<T>,
    len: usize,
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

impl<T: Ord> BinarySearchTree<T> {
    pub fn new() -> Self {
        Self { root: None, len: 0 }
    }

    fn remove_from(link: &mut Link<T>, value: &T) -> bool {
        let Some(node) = link.as_mut() else {
            return false;
        };
        match value.cmp(&node.value) {
            std::cmp::Ordering::Less => {
                return Self::remove_from(&mut node.left, value);
            }
            std::cmp::Ordering::Greater => {
                return Self::remove_from(&mut node.right, value);
            }
            std::cmp::Ordering::Equal => {}
        }

        if node.left.is_none() {
            *link = node.right.take();
        } else if node.right.is_none() {
            *link = node.left.take();
        } else {
            node.value = Self::take_min(&mut node.right);
        }
        true
    }

    //inorder successor, smallest of right subtree
    fn take_min(link: &mut Link<T>) -> T {
        if link.as_ref().unwrap().left.is_some() {
            return Self::take_min(&mut link.as_mut().unwrap().left);
        }

        let mut node = link.take().unwrap();
        *link = node.right.take();
        node.value
    }
}

impl<T: Ord> Collection for BinarySearchTree<T> {
    fn len(&self) -> usize {
        self.len
    }
}

impl<T: Ord> SearchTree<T> for BinarySearchTree<T> {
    type Node = Node<T>;
    fn root(&self) -> Option<&Self::Node> {
        self.root.as_deref()
    }
    fn insert(&mut self, value: T) -> bool {
        let mut link = &mut self.root;
        loop {
            match link {
                Some(node) => match value.cmp(&node.value) {
                    std::cmp::Ordering::Less => link = &mut node.left,
                    std::cmp::Ordering::Greater => link = &mut node.right,
                    std::cmp::Ordering::Equal => return false,
                },
                None => {
                    *link = Some(Box::new(Node {
                        value,
                        left: None,
                        right: None,
                    }));
                    self.len += 1;
                    return true;
                }
            }
        }
    }

    fn remove(&mut self, value: &T) -> bool {
        let removed = Self::remove_from(&mut self.root, value);
        if removed {
            self.len -= 1;
        }
        removed
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
