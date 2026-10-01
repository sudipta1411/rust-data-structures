use super::block::Block;
use super::slot::Slot;

pub struct Iter<'a, T> {
    blocks: std::slice::Iter<'a, Box<Block<T>>>,
    current: Option<std::slice::Iter<'a, Slot<T>>>,
    remaining: usize,
}

impl<'a, T> Iter<'a, T> {
    pub(crate) fn new(blocks: &'a [Box<Block<T>>], len: usize) -> Self {
        Self {
            blocks: blocks.iter(),
            current: None,
            remaining: len,
        }
    }
}

impl<'a, T> Iterator for Iter<'a, T> {
    type Item = &'a T;

    fn next(&mut self) -> Option<Self::Item> {
        loop {
            if let Some(slots) = &mut self.current {
                for slot in slots {
                    if let Some(value) = slot.value.as_ref() {
                        self.remaining -= 1;
                        return Some(value);
                    }
                }
            }
            let block = self.blocks.next()?;
            self.current = Some(block.slots().iter());
        }
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        (self.remaining, Some(self.remaining))
    }
}

impl<T> ExactSizeIterator for Iter<'_, T> {}

impl<T> std::iter::FusedIterator for Iter<'_, T> {}

pub struct IterMut<'a, T> {
    blocks: std::slice::IterMut<'a, Box<Block<T>>>,
    current: Option<std::slice::IterMut<'a, Slot<T>>>,
    remaining: usize,
}

impl<'a, T> IterMut<'a, T> {
    pub(crate) fn new(blocks: &'a mut [Box<Block<T>>], len: usize) -> Self {
        Self {
            blocks: blocks.iter_mut(),
            current: None,
            remaining: len,
        }
    }
}

impl<'a, T> Iterator for IterMut<'a, T> {
    type Item = &'a mut T;

    fn next(&mut self) -> Option<Self::Item> {
        loop {
            if let Some(slots) = &mut self.current {
                for slot in slots {
                    if let Some(value) = slot.value.as_mut() {
                        self.remaining -= 1;
                        return Some(value);
                    }
                }
            }
            let block = self.blocks.next()?;
            self.current = Some(block.slots_mut().iter_mut());
        }
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        (self.remaining, Some(self.remaining))
    }
}

impl<T> ExactSizeIterator for IterMut<'_, T> {}

impl<T> std::iter::FusedIterator for IterMut<'_, T> {}
