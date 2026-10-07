use block::Block;

pub use drain::Drain;
pub use entry::VacantEntry;
pub use handle::Handle;
pub use into_iter::IntoIter;
pub use iter::{Iter, IterMut};

mod block;
mod drain;
mod entry;
mod handle;
mod into_iter;
mod iter;
mod slot;

const DEFAULT_BLOCK_CAPACITY: usize = 256;

#[derive(Debug, Clone, Copy)]
struct Location {
    block: u32,
    slot: u32,
}

// Invariants:
//
// 1. Every location in `free` points to a vacant slot.
// 2. Every vacant slot occurs exactly once in `free`, unless
//    that slot is temporarily reserved by a live VacantEntry.
// 3. No occupied slot occurs in `free`.
// 4. Hive::len equals the number of occupied slots.
// 5. Block::len equals the number of occupied slots in that block.
// 6. Existing blocks and slot arrays are never resized.
// 7. A slot generation changes whenever its value is removed.
// 8. A Handle is valid only when its block, slot, and generation match an occupied slot.
// 9. A slot is occupied exactly when its occupancy bit is set.
// 10. If an occupancy bit is set, the corresponding Option<T>
//     is Some(T).
// 11. If an occupancy bit is clear, the corresponding Option<T>
//     is None.
pub struct Hive<T> {
    blocks: Vec<Box<Block<T>>>,
    free: Vec<Location>,
    len: usize,
    block_capacity: usize,
}

impl<T> Hive<T> {
    pub fn with_block_capacity(block_capacity: usize) -> Self {
        assert!(
            block_capacity > 0,
            "block capacity must be greater than zero"
        );
        assert!(
            block_capacity <= u32::MAX as usize,
            "block capacity exceeds u32 handle range"
        );
        Self {
            blocks: Vec::new(),
            free: Vec::new(),
            len: 0,
            block_capacity,
        }
    }

    pub fn new() -> Self {
        Self::with_block_capacity(DEFAULT_BLOCK_CAPACITY)
    }

    pub fn clear(&mut self) {
        //self.drain().for_each(drop);
        for blk_index in 0..self.blocks.len() {
            let mut slot_index = 0;
            while let Some(index) = self.blocks[blk_index].next_occupied(slot_index) {
                slot_index = index + 1;
                let generation = self.blocks[blk_index]
                    .slot(index)
                    .expect("occupied slot must exists")
                    .generation;
                let handle = Handle::new(blk_index, index, generation);
                let value = self
                    .remove(handle)
                    .expect("occupied slot must be removable");
                drop(value);
            }
        }
        debug_assert!(self.is_empty());
    }

    pub fn len(&self) -> usize {
        self.len
    }

    pub fn is_empty(&self) -> bool {
        self.len == 0
    }

    pub fn block_count(&self) -> usize {
        self.blocks.len()
    }

    pub fn capacity(&self) -> usize {
        self.blocks.iter().map(|block| block.capacity()).sum()
    }

    pub fn block_capacity(&self) -> usize {
        self.block_capacity
    }

    pub fn drain(&mut self) -> Drain<'_, T> {
        Drain::new(self)
    }

    pub fn insert(&mut self, value: T) -> Handle {
        let loc = self.reserve_location();
        self.insert_at(loc, value)
    }

    pub fn vacant_entry(&mut self) -> VacantEntry<'_, T> {
        VacantEntry::new(self)
    }

    pub fn get(&self, handle: Handle) -> Option<&T> {
        let blk = self.blocks.get(handle.block())?;
        if !blk.is_occupied(handle.slot()) {
            return None;
        }
        let slot = blk.slot(handle.slot())?;
        blk.debug_assert_consistent(handle.slot());
        if !slot.generation_matches(handle.generation()) {
            return None;
        }
        slot.value.as_ref()
    }

    pub fn get_mut(&mut self, handle: Handle) -> Option<&mut T> {
        let blk = self.blocks.get_mut(handle.block())?;
        if !blk.is_occupied(handle.slot()) {
            return None;
        }
        blk.debug_assert_consistent(handle.slot());
        let slot = blk.slot_mut(handle.slot())?;
        if !slot.generation_matches(handle.generation()) {
            return None;
        }
        slot.value.as_mut()
    }

    pub fn contains(&self, handle: Handle) -> bool {
        self.get(handle).is_some()
    }

    pub fn remove(&mut self, handle: Handle) -> Option<T> {
        let block = self.blocks.get_mut(handle.block())?;
        let value = block.remove(handle.slot(), handle.generation())?;
        self.free.push(Location {
            block: handle.block() as u32,
            slot: handle.slot() as u32,
        });
        self.len -= 1;
        Some(value)
    }

    pub fn iter(&self) -> Iter<'_, T> {
        Iter::new(&self.blocks, self.len)
    }

    pub fn iter_mut(&mut self) -> IterMut<'_, T> {
        IterMut::new(&mut self.blocks, self.len)
    }

    pub fn retain<F>(&mut self, mut keep: F)
    where
        F: FnMut(Handle, &mut T) -> bool,
    {
        for blk_index in 0..self.blocks.len() {
            let mut slot_index = 0;
            while let Some(index) = self.blocks[blk_index].next_occupied(slot_index) {
                slot_index = index + 1;
                let generation = self.blocks[blk_index]
                    .slot(index)
                    .expect("occupied slot must exist")
                    .generation;
                let handle = Handle::new(blk_index, index, generation);
                let keep_value = {
                    let blk = &mut self.blocks[blk_index];
                    blk.debug_assert_consistent(index);
                    let value = blk
                        .slot_mut(index)
                        .expect("occupied slot must exist")
                        .value
                        .as_mut()
                        .expect("occupied bit requires a value");
                    keep(handle, value)
                };
                if !keep_value {
                    let value = self
                        .remove(handle)
                        .expect("occupied slot must be removable");
                    drop(value);
                }
            }
        }
    }

    fn allocate_block(&mut self) {
        let block_index = self.blocks.len();
        assert!(block_index <= u32::MAX as usize, "hive has too many blocks");
        let block = Box::new(Block::<T>::new(self.block_capacity));
        let block_index = block_index as u32;
        for slot in (0..self.block_capacity).rev() {
            self.free.push(Location {
                block: block_index,
                slot: slot as u32,
            });
        }
        self.blocks.push(block);
    }

    pub(crate) fn reserve_location(&mut self) -> Location {
        if self.free.is_empty() {
            self.allocate_block();
        }
        self.free.pop().expect("a free location must exists")
    }

    pub(crate) fn insert_at(&mut self, location: Location, value: T) -> Handle {
        let blk = &mut self.blocks[location.block as usize];
        let generation = blk.insert(location.slot as usize, value);
        self.len += 1;
        Handle::new(location.block as usize, location.slot as usize, generation)
    }
}

impl<T> Default for Hive<T> {
    fn default() -> Self {
        Self::new()
    }
}

impl<'a, T> IntoIterator for &'a Hive<T> {
    type Item = &'a T;
    type IntoIter = Iter<'a, T>;

    fn into_iter(self) -> Self::IntoIter {
        self.iter()
    }
}

impl<'a, T> IntoIterator for &'a mut Hive<T> {
    type Item = &'a mut T;
    type IntoIter = IterMut<'a, T>;

    fn into_iter(self) -> Self::IntoIter {
        self.iter_mut()
    }
}

impl<T> IntoIterator for Hive<T> {
    type Item = T;
    type IntoIter = IntoIter<T>;

    fn into_iter(self) -> Self::IntoIter {
        IntoIter::new(self)
    }
}
