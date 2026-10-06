use crate::Handle;

use super::Hive;

pub struct Drain<'a, T> {
    hive: &'a mut Hive<T>,
    block_index: usize,
    slot_index: usize,
    remaining: usize,
}

impl<'a, T> Drain<'a, T> {
    pub(crate) fn new(hive: &'a mut Hive<T>) -> Self {
        let remaining = hive.len();
        Self {
            hive,
            block_index: 0,
            slot_index: 0,
            remaining,
        }
    }
}

impl<T> Iterator for Drain<'_, T> {
    type Item = T;

    fn next(&mut self) -> Option<Self::Item> {
        while self.block_index < self.hive.blocks.len() {
            let cap = self.hive.blocks[self.block_index].capacity();
            while self.slot_index < cap {
                let cur_slot = self.slot_index;
                self.slot_index += 1;
                let generation = {
                    let slot = self.hive.blocks[self.block_index]
                        .slot(cur_slot)
                        .expect("slot index is valid");
                    if slot.value.is_none() {
                        continue;
                    }
                    slot.generation
                };
                let handle = Handle::new(self.block_index, cur_slot, generation);
                let value = self
                    .hive
                    .remove(handle)
                    .expect("occupied slot must be removable");
                self.remaining -= 1;
                return Some(value);
            }
            self.block_index += 1;
            self.slot_index = 0;
        }
        None
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        (self.remaining, Some(self.remaining))
    }
}

impl<T> ExactSizeIterator for Drain<'_, T> {}

impl<T> std::iter::FusedIterator for Drain<'_, T> {}

impl<T> Drop for Drain<'_, T> {
    fn drop(&mut self) {
        while let Some(value) = self.next() {
            drop(value);
        }
    }
}
