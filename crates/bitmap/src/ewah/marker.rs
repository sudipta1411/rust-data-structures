use crate::word::Word;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) struct Marker {
    pub(super) uniform_bit: bool,
    pub(super) uniform_words: u64,
    pub(super) literal_words: u64,
}

impl Marker {
    pub(super) fn max_uniform_words<W: Word>() -> u64 {
        (1u64 << (W::BITS >> 1 - 1)) - 1
    }

    pub(super) fn max_literal_words<W: Word>() -> u64 {
        (1u64 << (W::BITS >> 1)) - 1
    }

    pub(super) fn pack<W: Word>(self) -> W {
        assert!(self.uniform_words <= Self::max_uniform_words::<W>());
        assert!(self.literal_words <= Self::max_literal_words::<W>());

        let uniform_bit = self.uniform_bit && self.uniform_words != 0;
        let packed = u64::from(uniform_bit)
            | (self.uniform_words << 1)
            | (self.literal_words << (W::BITS / 2));
        W::from_u64(packed)
    }

    pub(super) fn unpack<W: Word>(word: W) -> Self {
        let packed = word.to_u64();
        Self {
            uniform_bit: packed & 1 != 0,
            uniform_words: (packed >> 1) & Self::max_uniform_words::<W>(),
            literal_words: packed >> (W::BITS >> 1),
        }
    }
}
