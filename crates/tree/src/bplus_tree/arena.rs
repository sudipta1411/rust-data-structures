use super::node::{Node, NodeId};

#[derive(Debug)]
pub(crate) struct Arena<K, V> {
    nodes: Vec<Node<K, V>>,
}

impl<K, V> Arena<K, V> {
    pub(crate) fn new() -> Self {
        Self { nodes: Vec::new() }
    }

    pub(crate) fn alloc(&mut self, node: Node<K, V>) -> NodeId {
        let id = NodeId(self.nodes.len());
        self.nodes.push(node);
        id
    }

    pub(crate) fn get(&self, id: NodeId) -> &Node<K, V> {
        &self.nodes[id.0]
    }

    pub(crate) fn get_mut(&mut self, id: NodeId) -> &mut Node<K, V> {
        &mut self.nodes[id.0]
    }
}
