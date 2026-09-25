use super::marker::Marker;
use crate::word::Word;

pub(crate) struct Builder<W: Word> {
    encoded: Vec<W>,
    uniform_word: Option<W>,
    uniform_words: u64,
    literals: Vec<W>,
}

impl<W: Word> Builder<W> {
    pub(crate) fn new() -> Self {
        Self {
            encoded: Vec::new(),
            uniform_word: None,
            uniform_words: 0,
            literals: Vec::new(),
        }
    }
    pub(crate) fn push(&mut self, word: W) {
        if word.is_uniform() {
            self.push_uniform(word)
        } else {
            self.push_literal(word)
        }
    }
    fn push_uniform(&mut self, word: W) {
        if !self.literals.is_empty() {
            self.flush();
        }
        match self.uniform_word {
            Some(current) if current == word => self.uniform_words += 1,
            Some(_) => {
                self.flush();
                self.uniform_word = Some(word);
                self.uniform_words = 1;
            }
            None => {
                self.uniform_word = Some(word);
                self.uniform_words = 1;
            }
        }
        if self.uniform_words == Marker::max_uniform_words::<W>() {
            self.flush();
        }
    }
    fn push_literal(&mut self, word: W) {
        debug_assert!(!word.is_uniform());
        self.literals.push(word);
        if self.literals.len() as u64 == Marker::max_literal_words::<W>() {
            self.flush();
        }
    }
    fn flush(&mut self) {
        if self.uniform_words == 0 && self.literals.is_empty() {
            return;
        }
        let marker = Marker {
            uniform_bit: self.uniform_word.is_some_and(|word| word == W::ONES),
            uniform_words: self.uniform_words,
            literal_words: self.literals.len() as u64,
        };
        self.encoded.push(marker.pack::<W>());
        self.encoded.append(&mut self.literals);
        self.uniform_word = None;
        self.uniform_words = 0;
    }
    pub(crate) fn finish(mut self) -> Vec<W> {
        self.flush();
        self.encoded
    }
}

impl<W: Word> Default for Builder<W> {
    fn default() -> Self {
        Self::new()
    }
}
