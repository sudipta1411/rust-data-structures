use collection::collection::Collection;

use super::{arena::Arena, node::LeafNode, node::Node, node::NodeId};

#[derive(Debug)]
pub struct BPlusTree<K, V> {
    pub(crate) arena: Arena<K, V>,
    pub(crate) root: NodeId,
    pub(crate) order: usize,
    pub(crate) len: usize,
}

impl<K, V> BPlusTree<K, V> {
    pub fn new(order: usize) -> Self {
        assert!(order > 0, "order must be positive");
        assert!(order <= (usize::MAX - 2) / 2, "order is too large");
        let mut arena = Arena::new();
        let root = arena.alloc(Node::Leaf(LeafNode::empty()));
        Self {
            arena,
            root,
            order,
            len: 0,
        }
    }

    pub fn clear(&mut self) {
        *self = Self::new(self.order);
    }
}

impl<K, V> Collection for BPlusTree<K, V> {
    fn len(&self) -> usize {
        self.len
    }
}
