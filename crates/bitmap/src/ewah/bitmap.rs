use super::marker::Marker;

use super::{Decoder, Encoder};

use super::super::word::Word;

pub struct EwahBitmap<W: Word> {
    encoded: Vec<W>,
    bit_len: usize,
}

impl<W: Word> EwahBitmap<W> {
    pub fn new() -> Self {
        Self {
            encoded: Vec::new(),
            bit_len: 0,
        }
    }

    pub fn from_words(words: &[W]) -> Self {
        let bit_len = words
            .len()
            .checked_mul(W::BITS as usize)
            .expect("bitmap bit length exceeds usize");
        Self {
            encoded: Encoder::new(words).encode(),
            bit_len,
        }
    }

    pub fn to_words(&self) -> Vec<W> {
        let word_count = self.bit_len / W::BITS as usize;
        let mut output = vec![W::ZERO; word_count];
        Decoder::new(&self.encoded)
            .decode_into(&mut output)
            .expect("internally generated EWAH encoding is valid");
        output
    }

    pub fn encoded_words(&self) -> &[W] {
        &self.encoded
    }

    pub fn bit_len(&self) -> usize {
        self.bit_len
    }

    pub fn is_empty(&self) -> bool {
        self.bit_len == 0
    }

    pub fn count_ones(&self) -> u64 {
        let mut total = 0u64;
        let mut index = 0;

        while index < self.encoded.len() {
            let marker = Marker::unpack(self.encoded[index]);
            index += 1;
            if marker.uniform_bit {
                total += marker.uniform_words * u64::from(W::BITS)
            }
            let literal_count =
                usize::try_from(marker.literal_words).expect("internal literal count fits usize");
            let end = index + literal_count;
            for word in &self.encoded[index..end] {
                total += u64::from(word.count_ones());
            }
            index = end;
        }
        total
    }

    pub fn clear(&mut self) {
        self.encoded.clear();
        self.bit_len = 0;
    }

    fn combine<F>(&self, other: &Self, mut operation: F) -> Result<Self, LengthMismatch>
    where
        F: FnMut(W, W) -> W,
    {
        if self.bit_len != other.bit_len {
            return Err(LengthMismatch {
                left_bits: self.bit_len,
                right_bits: other.bit_len,
            });
        }

        let mut left = self.to_words();
        let right = other.to_words();

        for (a, b) in left.iter_mut().zip(right) {
            *a = operation(*a, b);
        }
        Ok(Self::from_words(&left))
    }

    pub fn and(&self, other: &Self) -> Result<Self, LengthMismatch> {
        self.combine(other, |a, b| a.bit_and(b))
    }

    pub fn or(&self, other: &Self) -> Result<Self, LengthMismatch> {
        self.combine(other, |a, b| a.bit_or(b))
    }

    pub fn xor(&self, other: &Self) -> Result<Self, LengthMismatch> {
        self.combine(other, |a, b| a.bit_xor(b))
    }

    pub fn and_not(&self, other: &Self) -> Result<Self, LengthMismatch> {
        self.combine(other, |a, b| a.bit_and(b.bit_not()))
    }

    pub fn not(&self) -> Self {
        let mut words = self.to_words();
        for word in &mut words {
            *word = word.bit_not();
        }
        Self::from_words(&words)
    }

    pub fn nand(&self, other: &Self) -> Result<Self, LengthMismatch> {
        self.combine(other, |a, b| a.bit_and(b).bit_not())
    }

    pub fn nor(&self, other: &Self) -> Result<Self, LengthMismatch> {
        self.combine(other, |a, b| a.bit_or(b).bit_not())
    }

    pub fn xnor(&self, other: &Self) -> Result<Self, LengthMismatch> {
        self.combine(other, |a, b| a.bit_xor(b).bit_not())
    }
}

impl<W: Word> Default for EwahBitmap<W> {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct LengthMismatch {
    pub left_bits: usize,
    pub right_bits: usize,
}

impl std::fmt::Display for LengthMismatch {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "bitmap length differ: {} and {} bits",
            self.left_bits, self.right_bits,
        )
    }
}

impl std::error::Error for LengthMismatch {}
