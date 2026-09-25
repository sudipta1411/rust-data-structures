use super::{ArrayStore, BITMAP_WORDS, WORD_BITS};

#[inline]
fn word_and_mask(value: u16) -> (usize, u64) {
    let value = value as usize;
    let word_index = value / WORD_BITS;
    let bit_index = value % WORD_BITS;
    (word_index, 1u64 << bit_index)
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct BitmapStore {
    words: Box<[u64; BITMAP_WORDS]>,
    cardinality: u32,
}

impl BitmapStore {
    pub(crate) fn new() -> Self {
        Self {
            words: Box::new([0; BITMAP_WORDS]),
            cardinality: 0,
        }
    }

    pub(crate) fn len(&self) -> u32 {
        self.cardinality
    }

    pub(crate) fn is_empty(&self) -> bool {
        self.cardinality == 0
    }

    pub(crate) fn contains(&self, value: u16) -> bool {
        let (word_index, mask) = word_and_mask(value);
        self.words[word_index] & mask != 0
    }

    pub(crate) fn insert(&mut self, value: u16) -> bool {
        let (word_index, mask) = word_and_mask(value);
        let word = &mut self.words[word_index];
        if *word & mask != 0 {
            return false;
        }
        *word |= mask;
        self.cardinality += 1;
        true
    }

    pub(crate) fn remove(&mut self, value: u16) -> bool {
        let (word_index, mask) = word_and_mask(value);
        let word = &mut self.words[word_index];

        if *word & mask == 0 {
            return false;
        }

        *word &= !mask;
        self.cardinality -= 1;
        true
    }

    pub(crate) fn insert_range(&mut self, start: u16, end: u16) -> u32 {
        let mut added = 0;
        for value in start..=end {
            added += u32::from(self.insert(value));
        }
        added
    }

    pub(crate) fn remove_range(&mut self, start: u16, end: u16) -> u32 {
        let mut removed = 0;
        for value in start..=end {
            removed += u32::from(self.remove(value));
        }
        removed
    }

    pub(crate) fn words(&self) -> &[u64; BITMAP_WORDS] {
        &self.words
    }

    pub(crate) fn from_words(words: Box<[u64; BITMAP_WORDS]>) -> Self {
        let cardinality = words.iter().map(|word| word.count_ones()).sum();
        Self { words, cardinality }
    }

    pub(crate) fn from_array(array: ArrayStore) -> Self {
        let card = array.len();
        let mut bitmap = Self::new();

        for value in array.into_values() {
            let (word_index, mask) = word_and_mask(value);
            bitmap.words[word_index] |= mask;
        }
        bitmap.cardinality = card;
        bitmap
    }

    pub(crate) fn into_array(self) -> ArrayStore {
        let mut values = Vec::with_capacity(self.cardinality as usize);
        for (word_index, orig_word) in self.words.iter().copied().enumerate() {
            let mut word = orig_word;
            while word != 0 {
                let bit_index = word.trailing_zeros() as usize;
                let value = word_index * WORD_BITS + bit_index;
                values.push(u16::try_from(value).expect("bitmap value must fit in u16"));
                word &= word - 1;
            }
        }
        ArrayStore::from_sorted_values(values)
    }

    pub(crate) fn iter(&self) -> BitmapIter<'_> {
        BitmapIter::new(&self.words, self.cardinality)
    }

    pub(crate) fn validate(&self) -> bool {
        let calc: u32 = self.words.iter().map(|word| word.count_ones()).sum();
        calc == self.cardinality
    }
}

impl Default for BitmapStore {
    fn default() -> Self {
        Self::new()
    }
}

pub(crate) struct BitmapIter<'a> {
    words: &'a [u64; BITMAP_WORDS],
    word_index: usize,
    current_word: u64,
    remaining: usize,
}

impl<'a> BitmapIter<'a> {
    fn new(words: &'a [u64; BITMAP_WORDS], cardinality: u32) -> Self {
        Self {
            words,
            word_index: 0,
            current_word: words[0],
            remaining: cardinality as usize,
        }
    }
}

impl Iterator for BitmapIter<'_> {
    type Item = u16;

    fn next(&mut self) -> Option<Self::Item> {
        loop {
            if self.current_word != 0 {
                let bit_index = self.current_word.trailing_zeros() as usize;

                // Clear the lowest set bit.
                self.current_word &= self.current_word - 1;
                self.remaining -= 1;

                let value = self.word_index * WORD_BITS + bit_index;

                return Some(u16::try_from(value).expect("bitmap value must fit in u16"));
            }

            self.word_index += 1;

            if self.word_index == BITMAP_WORDS {
                return None;
            }

            self.current_word = self.words[self.word_index];
        }
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        (self.remaining, Some(self.remaining))
    }
}

impl ExactSizeIterator for BitmapIter<'_> {
    fn len(&self) -> usize {
        self.remaining
    }
}

impl std::iter::FusedIterator for BitmapIter<'_> {}
