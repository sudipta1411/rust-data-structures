use crate::bplus_tree::node::{InternalNode, LeafNode};

use super::{node::Node, node::NodeId, tree::BPlusTree};

struct Split<K> {
    separator: K,
    right: NodeId,
}

struct InsertResult<K, V> {
    previous: Option<V>,
    split: Option<Split<K>>,
}

impl<K: Ord + Clone, V> BPlusTree<K, V> {
    pub fn insert(&mut self, key: K, value: V) -> Option<V> {
        let result = self.insert_into(self.root, key, value);
        if let Some(split) = result.split {
            let new_root = InternalNode {
                keys: vec![split.separator],
                children: vec![self.root, split.right],
            };
            self.root = self.arena.alloc(Node::Internal(new_root));
        }

        if result.previous.is_none() {
            self.len += 1;
        }
        result.previous
    }

    fn insert_into(&mut self, id: NodeId, key: K, value: V) -> InsertResult<K, V> {
        let dest = match self.arena.get(id) {
            Node::Leaf(_) => None,
            Node::Internal(node) => {
                let index = node.keys.partition_point(|separator| separator <= &key);
                Some((index, node.children[index]))
            }
        };
        let Some((child_index, child_id)) = dest else {
            return self.insert_leaf(id, key, value);
        };
        let result = self.insert_into(child_id, key, value);
        let Some(split) = result.split else {
            return InsertResult {
                previous: result.previous,
                split: None,
            };
        };

        let overflow = {
            let Node::Internal(node) = self.arena.get_mut(id) else {
                unreachable!("expected internal node");
            };
            node.keys.insert(child_index, split.separator);
            node.children.insert(child_index + 1, split.right);
            node.keys.len() > 2 * self.order
        };

        let split = if overflow {
            Some(self.split_internal(id))
        } else {
            None
        };
        InsertResult {
            previous: result.previous,
            split,
        }
    }

    fn insert_leaf(&mut self, id: NodeId, key: K, value: V) -> InsertResult<K, V> {
        let overflow = {
            let Node::Leaf(leaf) = self.arena.get_mut(id) else {
                unreachable!("expected a leaf");
            };
            match leaf
                .entries
                .binary_search_by(|(stored, _)| stored.cmp(&key))
            {
                Ok(index) => {
                    let previous = std::mem::replace(&mut leaf.entries[index].1, value);
                    return InsertResult {
                        previous: Some(previous),
                        split: None,
                    };
                }
                Err(index) => {
                    leaf.entries.insert(index, (key, value));
                }
            }
            leaf.entries.len() > 2 * self.order
        };
        let split = if overflow {
            Some(self.split_leaf(id))
        } else {
            None
        };
        InsertResult {
            previous: None,
            split,
        }
    }

    fn split_leaf(&mut self, id: NodeId) -> Split<K> {
        let (separator, right_entries, old_next) = {
            let Node::Leaf(leaf) = self.arena.get_mut(id) else {
                unreachable!("expected a leaf");
            };
            let separator = leaf.entries[self.order].0.clone();
            let right_entries = leaf.entries.split_off(self.order);
            (separator, right_entries, leaf.next)
        };
        let right = self.arena.alloc(Node::Leaf(LeafNode {
            entries: right_entries,
            next: old_next,
        }));
        let Node::Leaf(leaf) = self.arena.get_mut(id) else {
            unreachable!("expected a leaf");
        };
        leaf.next = Some(right);
        Split { separator, right }
    }

    fn split_internal(&mut self, id: NodeId) -> Split<K> {
        let (separator, right_keys, right_children) = {
            let Node::Internal(node) = self.arena.get_mut(id) else {
                unreachable!("expected internal node");
            };
            let right_keys = node.keys.split_off(self.order + 1);
            let separator = node.keys.pop().unwrap();
            let right_children = node.children.split_off(self.order + 1);
            (separator, right_keys, right_children)
        };
        let right = self.arena.alloc(Node::Internal(InternalNode {
            keys: right_keys,
            children: right_children,
        }));
        Split { separator, right }
    }
}
