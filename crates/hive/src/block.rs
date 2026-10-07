use super::slot::Slot;

const WORD_BITS: usize = u64::BITS as usize;

fn word_and_mask(index: usize) -> (usize, u64) {
    let word = index / WORD_BITS;
    let bit = index % WORD_BITS;
    (word, 1u64 << bit)
}

pub(crate) struct Block<T> {
    slots: Box<[Slot<T>]>,
    occupied: Box<[u64]>,
    len: usize,
}

impl<T> Block<T> {
    pub(crate) fn new(capacity: usize) -> Self {
        let slots = std::iter::repeat_with(Slot::vacant)
            .take(capacity)
            .collect::<Vec<_>>()
            .into_boxed_slice();

        let word_cnt = capacity.div_ceil(WORD_BITS);
        let occupied = vec![0u64; word_cnt].into_boxed_slice();

        Self {
            slots,
            occupied,
            len: 0,
        }
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
        assert!(index < self.slots.len(), "slot index out of bounds");
        self.debug_assert_consistent(index);
        assert!(
            !self.is_occupied(index),
            "attempted insertion into occupied slot"
        );
        self.slots[index].value = Some(value);
        self.set_occupied(index);
        self.len += 1;
        self.debug_assert_consistent(index);
        self.slots[index].generation
    }

    pub(crate) fn remove(&mut self, index: usize, generation: u64) -> Option<T> {
        if index >= self.slots.len() {
            return None;
        }
        self.debug_assert_consistent(index);
        if !self.is_occupied(index) {
            return None;
        }
        if self.slots[index].generation != generation {
            return None;
        }
        self.clear_occupied(index);
        let slot = &mut self.slots[index];
        let value = slot
            .value
            .take()
            .expect("occupied bit requires an initialized value");
        slot.generation = slot.generation.wrapping_add(1);
        self.len -= 1;
        self.debug_assert_consistent(index);
        Some(value)
    }

    pub(crate) fn slots(&self) -> &[Slot<T>] {
        &self.slots
    }

    pub(crate) fn slots_mut(&mut self) -> &mut [Slot<T>] {
        &mut self.slots
    }

    pub(crate) fn occupied_words(&self) -> &[u64] {
        &self.occupied
    }

    pub(crate) fn is_occupied(&self, index: usize) -> bool {
        if index >= self.slots.len() {
            return false;
        }

        let (word, mask) = word_and_mask(index);
        self.occupied[word] & mask != 0
    }

    pub(crate) fn debug_assert_consistent(&self, index: usize) {
        debug_assert_eq!(
            self.is_occupied(index),
            self.slots[index].value.is_some(),
            "occupancy bitmap disagrees with slot value"
        );
    }

    pub(crate) fn debug_assert_all_consistent(&self) {
        let mut occupied_cnt = 0;
        for index in 0..self.slots.len() {
            self.debug_assert_consistent(index);
            if self.is_occupied(index) {
                occupied_cnt += 1;
            }
        }
        debug_assert_eq!(
            occupied_cnt, self.len,
            "block length disagrees with occupancy bitmap"
        );
    }

    pub(crate) fn next_occupied(&self, from: usize) -> Option<usize> {
        if from >= self.slots.len() {
            return None;
        }
        let mut word_index = from / WORD_BITS;
        let bit_index = from % WORD_BITS;

        let mut word = self.occupied[word_index];
        word &= u64::MAX << bit_index;
        loop {
            if word != 0 {
                let bit = word.trailing_zeros() as usize;
                let index = word_index * WORD_BITS + bit;
                if index < self.slots.len() {
                    return Some(index);
                }
                return None;
            }
            word_index += 1;
            if word_index >= self.occupied.len() {
                return None;
            }
            word = self.occupied[word_index];
        }
    }

    fn set_occupied(&mut self, index: usize) {
        let (word, mask) = word_and_mask(index);
        self.occupied[word] |= mask;
    }

    fn clear_occupied(&mut self, index: usize) {
        let (word, mask) = word_and_mask(index);
        self.occupied[word] &= !mask;
    }
}
