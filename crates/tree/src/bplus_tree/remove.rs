use super::{node::Node, tree::BPlusTree};

impl<K: Ord, V> BPlusTree<K, V> {
    pub fn remove(&mut self, key: &K) -> Option<V> {
        let id = self.find_leaf(key);
        let Node::Leaf(leaf) = self.arena.get_mut(id) else {
            unreachable!("find_leaf must return a leaf");
        };
        let index = leaf
            .entries
            .binary_search_by(|(stored, _)| stored.cmp(key))
            .ok()?;
        let (_, value) = leaf.entries.remove(index);
        self.len -= 1;
        Some(value)
    }
}
