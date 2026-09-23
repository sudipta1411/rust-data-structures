use bst::{avl::AvlTree, bst::BinarySearchTree};
use collection::{
    collection::{Collection, SearchTree},
    general_tree::GeneralTree,
    tree_node::TreeNode,
};
use queue::queue::Queue;
use stack::stack::Stack;
use std::{cmp::Ordering, collections::BTreeMap};
use tree::{bplus_tree::BPlusTree, tree::Tree};

fn main() {
    let v = vec!["Hello1", "Hello2", "Hello3", "Hello4"];
    let mut stack: Stack<&str> = Stack::new();
    for item in &v {
        stack.push(item);
    }
    println!("\n----- STACK -----\n");
    println!("Stack : {:?}", stack);
    println!("Top : {:?}", stack.peek());
    println!("Top removed : {:?}", stack.pop());
    println!("Stack length: {:?}", stack.len());
    println!("Stack is empty: {:?}", stack.is_empty());

    println!("\n----- QUEUE -----\n");
    let mut queue: Queue<&str> = Queue::new();
    for item in &v {
        queue.enqueue(item);
    }
    println!("Queue : {:?}", queue);
    println!("Front : {:?}", queue.peek());
    println!("Front removed : {:?}", queue.dequeue());
    println!("Queue length: {:?}", queue.len());
    println!("Queue is empty: {:?}", queue.is_empty());

    println!("\n----- BINARY SEARCH TREE -----\n");
    //let mut tree = BinarySearchTree::new();
    let mut tree = AvlTree::new();
    for user in [
        User::new("Alice", 30),
        User::new("Bob", 20),
        User::new("Charlie", 40),
        User::new("Diana", 25),
        User::new("Sneha", 26),
        User::new("Dissing", 29),
        User::new("Humphry", 21),
        User::new("ELizabeth", 67),
    ] {
        tree.insert(user);
    }
    println!("len of the bst : {}", tree.len());
    println!("bst : {:?}", tree);
    let target = User::new("", 30);
    println!("tree contains : {}", tree.contains(&target));
    let mut v = tree.inorder_traverse();
    println!("inorder : {:?}", v);
    v = tree.preorder_traverse();
    println!("preorder : {:?}", v);
    v = tree.postorder_traverse();
    println!("postorder : {:?}", v);
    println!("height : {}", tree.height());
    println!(
        "min : {:?}, max : {:?}",
        tree.min().unwrap(),
        tree.max().unwrap()
    );
    tree.clear();
    println!("is tree empty {}", tree.is_empty());

    println!("\n----- TREE -----\n");
    let mut tree = Tree::with_root(User::new("Alice", 20));
    {
        let root = tree.root_mut().unwrap();
        let bob = root.add_child(User::new("Bob", 30));
        bob.add_child(User::new("Diana", 23));
        bob.add_child(User::new("Eve", 24));
        bob.add_child(User::new("Clarence", 35));
        root.add_child(User::new("Charlie", 34));
    }
    println!("tree {:?}", tree);
    println!("tree length : {}, height : {}", tree.len(), tree.height());
    println!("level order : {:?}", tree.level_order_traverse());
    println!("contains : {}", tree.contains(|user| user.name == "Dian"));
    tree.clear();
    println!("is empty : {}", tree.is_empty());

    check_bplus_tree();
}

fn check_bplus_tree() {
    for order in 1..300 {
        let mut tree = BPlusTree::new(order);
        let mut expected = BTreeMap::new();

        for key in (0..100).rev() {
            assert_eq!(tree.insert(key, 10 * key), expected.insert(key, 10 * key));
        }

        assert_eq!(tree.len(), expected.len());

        for key in 0..100 {
            assert_eq!(tree.get(&key), expected.get(&key));
        }
        assert_eq!(tree.insert(50, 999), expected.insert(50, 999));

        for key in (0..100).step_by(2) {
            assert_eq!(tree.remove(&key), expected.remove(&key));
        }

        for key in 0..100 {
            assert_eq!(tree.get(&key), expected.get(&key));
        }
        for key in 0..100 {
            assert_eq!(tree.remove(&key), expected.remove(&key));
        }
        assert!(tree.is_empty());
        for key in 0..100 {
            assert_eq!(tree.insert(key, key), expected.insert(key, key),);
        }

        for key in 0..100 {
            assert_eq!(tree.get(&key), expected.get(&key));
        }

        assert_eq!(tree.len(), expected.len());
    }
    println!("B+ tree map checks passed.");
}

#[derive(Debug)]
struct User {
    name: String,
    age: u32,
}

impl User {
    fn new(name: &str, age: u32) -> Self {
        Self {
            name: name.to_owned(),
            age,
        }
    }
}

impl PartialEq for User {
    fn eq(&self, other: &Self) -> bool {
        self.age == other.age
    }
}

impl Eq for User {}

impl PartialOrd for User {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for User {
    fn cmp(&self, other: &Self) -> Ordering {
        self.age.cmp(&other.age)
    }
}
