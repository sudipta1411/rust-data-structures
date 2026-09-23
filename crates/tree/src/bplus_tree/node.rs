#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct NodeId(pub(crate) usize);

#[derive(Debug)]
pub(crate) struct LeafNode<K, V> {
    pub(crate) entries: Vec<(K, V)>,
    pub(crate) next: Option<NodeId>,
}

#[derive(Debug)]
pub(crate) struct InternalNode<K> {
    pub(crate) keys: Vec<K>,
    pub(crate) children: Vec<NodeId>,
}

#[derive(Debug)]
pub(crate) enum Node<K, V> {
    Leaf(LeafNode<K, V>),
    Internal(InternalNode<K>),
}

impl<K, V> LeafNode<K, V> {
    pub(crate) fn empty() -> Self {
        Self {
            entries: Vec::new(),
            next: None,
        }
    }
}
