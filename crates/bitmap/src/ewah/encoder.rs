use super::{super::word::Word, marker::Marker};

pub struct Encoder<'a, W: Word> {
    source: &'a [W],
}

impl<'a, W: Word> Encoder<'a, W> {
    pub fn new(source: &'a [W]) -> Self {
        Self { source }
    }

    pub fn encode(&self) -> Vec<W> {
        let mut encoded = Vec::new();
        let mut index = 0;

        let max_uniform = Marker::max_uniform_words::<W>();
        let max_literals = Marker::max_literal_words::<W>();
        while index < self.source.len() {
            let mut uniform_bit = false;
            let mut uniform_words = 0u64;
            let first = self.source[index];

            if first.is_uniform() {
                uniform_bit = first == W::ONES;
                while index < self.source.len()
                    && self.source[index] == first
                    && uniform_words < max_uniform
                {
                    uniform_words += 1;
                    index += 1;
                }
                let literal_start = index;
                let mut literal_words = 0u64;
                while index < self.source.len()
                    && !self.source[index].is_uniform()
                    && literal_words < max_literals
                {
                    literal_words += 1;
                    index += 1;
                }
                let marker = Marker {
                    uniform_bit,
                    uniform_words,
                    literal_words,
                };
                encoded.push(marker.pack::<W>());
                encoded.extend_from_slice(&self.source[literal_start..index]);
            }
        }
        encoded
    }
}
