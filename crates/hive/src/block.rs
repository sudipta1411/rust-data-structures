use super::slot::Slot;

pub(crate) struct Block<T> {
    slots: Box<[Slot<T>]>,
    len: usize,
}

impl<T> Block<T> {
    pub(crate) fn new(capacity: usize) -> Self {
        let slots = std::iter::repeat_with(Slot::vacant)
            .take(capacity)
            .collect::<Vec<_>>()
            .into_boxed_slice();

        Self { slots, len: 0 }
    }

    pub(crate) fn capacity(&self) -> usize {
        self.slots.len()
    }

    pub(crate) fn slot(&self, index: usize) -> Option<&Slot<T>> {
        self.slots.get(index)
    }

    pub(crate) fn slot_mut(&mut self, index: usize) -> Option<&mut Slot<T>> {
        self.slots.get_mut(index)
    }

    pub(crate) fn insert(&mut self, index: usize, value: T) -> u64 {
        let slot = &mut self.slots[index];
        debug_assert!(slot.value.is_none());
        slot.value = Some(value);
        self.len += 1;
        slot.generation
    }

    pub(crate) fn remove(&mut self, index: usize, generation: u64) -> Option<T> {
        let slot = self.slots.get_mut(index)?;
        if !slot.matches(generation) {
            return None;
        }

        let value = slot.value.take()?;
        slot.generation = slot.generation.wrapping_add(1);
        self.len -= 1;
        Some(value)
    }

    pub(crate) fn slots(&self) -> &[Slot<T>] {
        &self.slots
    }

    pub(crate) fn slots_mut(&mut self) -> &mut [Slot<T>] {
        &mut self.slots
    }
}
