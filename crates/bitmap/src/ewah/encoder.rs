use super::{super::word::Word, builder::Builder};

pub struct Encoder<'a, W: Word> {
    source: &'a [W],
}
impl<'a, W: Word> Encoder<'a, W> {
    pub fn new(source: &'a [W]) -> Self {
        Self { source }
    }
    pub fn encode(&self) -> Vec<W> {
        let mut builder = Builder::new();
        for &word in self.source {
            builder.push(word)
        }
        builder.finish()
    }
}
