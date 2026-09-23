use collection::collection::Collection;
use std::collections::VecDeque;

#[derive(Debug)]
pub struct Queue<T> {
    items: VecDeque<T>,
}

impl<T> Queue<T> {
    pub fn new() -> Self {
        Self {
            items: VecDeque::new(),
        }
    }

    pub fn enqueue(&mut self, item: T) {
        self.items.push_back(item);
    }

    pub fn dequeue(&mut self) -> Option<T> {
        self.items.pop_front()
    }

    pub fn peek(&self) -> Option<&T> {
        self.items.front()
    }
}

impl<T> Collection for Queue<T> {
    fn len(&self) -> usize {
        self.items.len()
    }
}
