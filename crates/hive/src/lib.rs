use block::Block;
use handle::Handle;
use iter::{Iter, IterMut};

mod block;
mod handle;
mod iter;
mod slot;

const DEFAULT_BLOCK_CAPACITY: usize = 256;

#[derive(Debug, Clone, Copy)]
struct Location {
    block: u32,
    slot: u32,
}

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
        self.free.clear();
        for (block_index, block) in self.blocks.iter_mut().enumerate() {
            for (slot_index, slot) in block.slots_mut().iter_mut().enumerate() {
                if slot.value.take().is_some() {
                    slot.generation = slot.generation.wrapping_add(1);
                }
                self.free.push(Location {
                    block: block_index as u32,
                    slot: slot_index as u32,
                });
            }
            block.reset_len();
        }
        self.len = 0;
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

    pub fn insert(&mut self, value: T) -> Handle {
        if self.free.is_empty() {
            self.allocate_block();
        }
        let location = self
            .free
            .pop()
            .expect("a newly allocated block has free slots");
        let block = &mut self.blocks[location.block as usize];
        let generation = block.insert(location.slot as usize, value);
        self.len += 1;
        Handle {
            block: location.block,
            slot: location.slot,
            generation,
        }
    }

    pub fn get(&self, handle: Handle) -> Option<&T> {
        let slot = self.blocks.get(handle.block())?.slot(handle.slot())?;
        if !slot.matches(handle.generation) {
            return None;
        }
        slot.value.as_ref()
    }

    pub fn get_mut(&mut self, handle: Handle) -> Option<&mut T> {
        let slot = self
            .blocks
            .get_mut(handle.block())?
            .slot_mut(handle.slot())?;
        if !slot.matches(handle.generation) {
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
