use super::{node::Node, node::NodeId, tree::BPlusTree};

impl<K: Ord, V> BPlusTree<K, V> {
    pub(crate) fn find_leaf(&self, key: &K) -> NodeId {
        let mut id = self.root;
        loop {
            match self.arena.get(id) {
                Node::Leaf(_) => return id,
                Node::Internal(node) => {
                    let index = node.keys.partition_point(|separator| separator <= key);
                    id = node.children[index];
                }
            }
        }
    }

    pub fn get(&self, key: &K) -> Option<&V> {
        let id = self.find_leaf(key);
        let Node::Leaf(leaf) = self.arena.get(id) else {
            unreachable!("find_leaf returned an internal node");
        };
        let index = leaf
            .entries
            .binary_search_by(|(stored, _)| stored.cmp(key))
            .ok()?;
        Some(&leaf.entries[index].1)
    }

    pub fn get_mut(&mut self, key: &K) -> Option<&mut V> {
        let id = self.find_leaf(key);
        let Node::Leaf(leaf) = self.arena.get_mut(id) else {
            unreachable!("find_leaf returned an internal node");
        };
        let index = leaf
            .entries
            .binary_search_by(|(stored, _)| stored.cmp(key))
            .ok()?;
        Some(&mut leaf.entries[index].1)
    }

    pub fn contains_key(&self, key: &K) -> bool {
        self.get(key).is_some()
    }
}
