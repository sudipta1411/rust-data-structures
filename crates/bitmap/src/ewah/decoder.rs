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
