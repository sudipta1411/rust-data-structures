use collection::collection::Collection;

#[derive(Debug)]
pub struct MinHeap<T> {
    items: Vec<T>,
}

impl<T: Ord> MinHeap<T> {
    pub fn new() -> Self {
        Self { items: Vec::new() }
    }

    pub fn from_vec(items: Vec<T>) -> Self {
        let mut heap = Self { items };
        for index in (0..heap.items.len() / 2).rev() {
            heap.sift_down(index);
        }
        heap
    }
    pub fn push(&mut self, value: T) {
        self.items.push(value);
        self.sift_up(self.items.len() - 1);
    }
    pub fn pop(&mut self) -> Option<T> {
        if self.items.is_empty() {
            return None;
        }
        let minimum = self.items.swap_remove(0);
        if !self.items.is_empty() {
            self.sift_down(0);
        }
        Some(minimum)
    }
    pub fn peek(&self) -> Option<&T> {
        self.items.first()
    }
    pub fn clear(&mut self) {
        self.items.clear();
    }

    fn sift_up(&mut self, mut index: usize) {
        while index > 0 {
            let parent = (index - 1) >> 1;
            if self.items[parent] <= self.items[index] {
                break;
            }
            self.items.swap(parent, index);
            index = parent;
        }
    }
    fn sift_down(&mut self, mut index: usize) {
        let len = self.items.len();
        while index < len / 2 {
            let left = index << 1 + 1;
            let right = left + 1;
            let mut smallest = left;
            if right < len && self.items[right] < self.items[left] {
                smallest = right;
            }
            if self.items[index] <= self.items[smallest] {
                break;
            }
            self.items.swap(index, smallest);
            index = smallest;
        }
    }
}

impl<T> Collection for MinHeap<T> {
    fn len(&self) -> usize {
        self.items.len()
    }
}
