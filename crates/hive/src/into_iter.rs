use super::{Handle, Hive};

pub struct IntoIter<T> {
    hive: Hive<T>,
    block_index: usize,
    slot_index: usize,
    remaining: usize,
}

impl<T> IntoIter<T> {
    pub(crate) fn new(hive: Hive<T>) -> Self {
        let remaining = hive.len();

        Self {
            hive,
            block_index: 0,
            slot_index: 0,
            remaining,
        }
    }
}

impl<T> Iterator for IntoIter<T> {
    type Item = T;

    fn next(&mut self) -> Option<Self::Item> {
        while self.block_index < self.hive.blocks.len() {
            let occupied = self.hive.blocks[self.block_index].next_occupied(self.slot_index);

            let Some(slot_index) = occupied else {
                self.block_index += 1;
                self.slot_index = 0;
                continue;
            };

            self.slot_index = slot_index + 1;

            let generation = self.hive.blocks[self.block_index]
                .slot(slot_index)
                .expect("occupied slot must exist")
                .generation;

            let handle = Handle::new(self.block_index, slot_index, generation);

            let value = self
                .hive
                .remove(handle)
                .expect("occupied slot must be removable");

            self.remaining -= 1;

            return Some(value);
        }

        None
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        (self.remaining, Some(self.remaining))
    }
}

impl<T> ExactSizeIterator for IntoIter<T> {}

impl<T> std::iter::FusedIterator for IntoIter<T> {}
