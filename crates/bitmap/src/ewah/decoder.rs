use std::fmt;

use crate::ewah::marker::Marker;

use super::super::word::Word;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DecodeError {
    CountTooLarge,
    TruncatedLiterals,
    OutputTooSmall,
    OutputLengthMismatch,
}

impl fmt::Display for DecodeError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let message = match self {
            Self::CountTooLarge => "marker count cannot fit in usize",
            Self::TruncatedLiterals => "marker reference missing literal words",
            Self::OutputTooSmall => "decoded words exceed output capacity",
            Self::OutputLengthMismatch => "decoded words do not fill the output",
        };
        f.write_str(message)
    }
}

impl std::error::Error for DecodeError {}

pub struct Decoder<'a, W> {
    encoded: &'a [W],
}

pub(crate) struct WordIter<'a, W: Word> {
    encoded: &'a [W],
    source_index: usize,
    uniform_value: W,
    uniform_remaining: usize,
    literal_remaining: usize,
}

impl<'a, W: Word> WordIter<'a, W> {
    pub(crate) fn new(encoded: &'a [W]) -> Self {
        Self {
            encoded,
            source_index: 0,
            uniform_value: W::ZERO,
            uniform_remaining: 0,
            literal_remaining: 0,
        }
    }
    fn load_marker(&mut self) -> bool {
        let Some(&word) = self.encoded.get(self.source_index) else {
            return false;
        };
        self.source_index += 1;
        let marker = Marker::unpack(word);
        self.uniform_value = if marker.uniform_bit { W::ONES } else { W::ZERO };
        self.uniform_remaining =
            usize::try_from(marker.uniform_words).expect("uniform count must fit usize");
        self.literal_remaining =
            usize::try_from(marker.literal_words).expect("literal count must fit usize");
        true
    }
}

impl<W: Word> Iterator for WordIter<'_, W> {
    type Item = W;
    fn next(&mut self) -> Option<W> {
        loop {
            if self.uniform_remaining != 0 {
                self.uniform_remaining -= 1;
                return Some(self.uniform_value);
            }
            if self.literal_remaining != 0 {
                let word = *self
                    .encoded
                    .get(self.source_index)
                    .expect("internally generated EWAH is valid");
                self.source_index += 1;
                self.literal_remaining -= 1;
                return Some(word);
            }
            if !self.load_marker() {
                return None;
            }
        }
    }
}
impl<W: Word> std::iter::FusedIterator for WordIter<'_, W> {}

impl<'a, W: Word> Decoder<'a, W> {
    pub fn new(encoded: &'a [W]) -> Self {
        Self { encoded }
    }

    fn marker_counts(word: W) -> Result<(bool, usize, usize), DecodeError> {
        let marker = Marker::unpack(word);
        let uniform_words =
            usize::try_from(marker.uniform_words).map_err(|_| DecodeError::CountTooLarge)?;
        let literal_words =
            usize::try_from(marker.literal_words).map_err(|_| DecodeError::CountTooLarge)?;
        Ok((marker.uniform_bit, uniform_words, literal_words))
    }

    fn validate(&self, expected_words: usize) -> Result<(), DecodeError> {
        let mut index = 0;
        let mut remaining = expected_words;

        while index < self.encoded.len() {
            let (_, uniform_words, literal_words) = Self::marker_counts(self.encoded[index])?;
            index += 1;
            if literal_words > self.encoded.len() - index {
                return Err(DecodeError::TruncatedLiterals);
            }
            remaining = remaining
                .checked_sub(uniform_words)
                .ok_or(DecodeError::OutputTooSmall)?;
            remaining = remaining
                .checked_sub(literal_words)
                .ok_or(DecodeError::OutputTooSmall)?;
            index += literal_words;
        }
        if remaining != 0 {
            return Err(DecodeError::OutputLengthMismatch);
        }
        Ok(())
    }

    pub fn decode_into(&self, output: &mut [W]) -> Result<(), DecodeError> {
        self.validate(output.len())?;
        let mut source_index = 0;
        let mut output_index = 0;
        while source_index < self.encoded.len() {
            let (uniform_bit, uniform_words, literal_words) =
                Self::marker_counts(self.encoded[source_index])?;
            source_index += 1;
            let uniform_value = if uniform_bit { W::ONES } else { W::ZERO };
            let run_end = output_index + uniform_words;
            output[output_index..run_end].fill(uniform_value);
            output_index = run_end;
            let source_end = source_index + literal_words;
            let output_end = output_index + literal_words;
            output[output_index..output_end]
                .copy_from_slice(&self.encoded[source_index..source_end]);
            source_index = source_end;
            output_index = output_end;
        }
        Ok(())
    }
}
