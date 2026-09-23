pub trait Collection {
    fn len(&self) -> usize;
    fn is_empty(&self) -> bool {
        self.len() == 0
    }
}

pub trait SearchNode<T> {
    fn value(&self) -> &T;
    fn left(&self) -> Option<&Self>;
    fn right(&self) -> Option<&Self>;
}

pub trait SearchTree<T: Ord>: Collection {
    type Node: SearchNode<T>;

    fn root(&self) -> Option<&Self::Node>;
    fn insert(&mut self, value: T) -> bool;
    fn remove(&mut self, value: &T) -> bool;
    fn clear(&mut self);

    fn min(&self) -> Option<&T> {
        let mut node = self.root()?;
        while let Some(left) = node.left() {
            node = left;
        }
        Some(node.value())
    }

    fn max(&self) -> Option<&T> {
        let mut node = self.root()?;
        while let Some(right) = node.right() {
            node = right;
        }
        Some(node.value())
    }

    fn height(&self) -> usize {
        let mut height = 0;
        let mut stack = Vec::new();
        if let Some(root) = self.root() {
            stack.push((root, 1));
        }
        while let Some((node, depth)) = stack.pop() {
            height = height.max(depth);
            if let Some(left) = node.left() {
                stack.push((left, depth + 1));
            }
            if let Some(right) = node.right() {
                stack.push((right, depth + 1));
            }
        }
        height
    }

    fn successor(&self, value: &T) -> Option<&T> {
        let mut cur = self.root();
        let mut candidate = None;
        while let Some(node) = cur {
            if node.value() > value {
                candidate = Some(node.value());
                cur = node.left();
            } else {
                cur = node.right();
            }
        }
        candidate
    }

    fn predecessor(&self, value: &T) -> Option<&T> {
        let mut cur = self.root();
        let mut candidate = None;
        while let Some(node) = cur {
            if node.value() < value {
                candidate = Some(node.value());
                cur = node.right();
            } else {
                cur = node.left();
            }
        }
        candidate
    }

    fn contains(&self, value: &T) -> bool {
        let mut cur = self.root();
        while let Some(node) = cur {
            match value.cmp(node.value()) {
                std::cmp::Ordering::Less => cur = node.left(),
                std::cmp::Ordering::Greater => cur = node.right(),
                std::cmp::Ordering::Equal => return true,
            }
        }
        false
    }

    fn travserse(&self, traversal_type: TraversalType) -> Option<Vec<&T>> {
        match traversal_type {
            TraversalType::InOrder => Some(self.inorder_traverse()),
            TraversalType::PostOrder => Some(self.postorder_traverse()),
            TraversalType::PreOrder => Some(self.preorder_traverse()),
            TraversalType::LevelOrder => None,
        }
    }

    fn inorder_traverse(&self) -> Vec<&T> {
        let mut values = Vec::with_capacity(self.len());
        let mut cur = self.root();
        let mut stack = Vec::new();
        while cur.is_some() || !stack.is_empty() {
            while let Some(node) = cur {
                stack.push(node);
                cur = node.left();
            }
            let node = stack.pop().unwrap();
            values.push(node.value());
            cur = node.right();
        }
        values
    }

    fn preorder_traverse(&self) -> Vec<&T> {
        let mut values = Vec::with_capacity(self.len());
        let mut stack = Vec::new();
        if let Some(root) = self.root() {
            stack.push(root);
        }
        while let Some(node) = stack.pop() {
            values.push(node.value());
            if let Some(right) = node.right() {
                stack.push(right);
            }
            if let Some(left) = node.left() {
                stack.push(left);
            }
        }
        values
    }

    fn postorder_traverse(&self) -> Vec<&T> {
        let mut values = Vec::with_capacity(self.len());
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
            if let Some(right) = node.right() {
                stack.push((right, false));
            }
            if let Some(left) = node.left() {
                stack.push((left, false));
            }
        }
        values
    }
}

#[derive(Clone, Copy)]
pub enum TraversalType {
    InOrder,
    PreOrder,
    PostOrder,
    LevelOrder,
}
