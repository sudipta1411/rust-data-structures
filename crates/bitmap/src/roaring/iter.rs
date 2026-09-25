use std::iter::FusedIterator;
use std::slice;

use super::container::Container;
use super::store::StoreIter;
use super::util::join;

pub struct Iter<'a> {
    containers: slice::Iter<'a, Container>,
    current_key: u16,
    current_values: Option<StoreIter<'a>>,
    remaining: u64,
}

impl<'a> Iter<'a> {
    pub(crate) fn new(containers: &'a [Container], cardinality: u64) -> Self {
        Self {
            containers: containers.iter(),
            current_key: 0,
            current_values: None,
            remaining: cardinality,
        }
    }
}

impl Iterator for Iter<'_> {
    type Item = u32;

    fn next(&mut self) -> Option<Self::Item> {
        loop {
            if let Some(values) = &mut self.current_values {
                if let Some(low) = values.next() {
                    self.remaining -= 1;
                    return Some(join(self.current_key, low));
                }
            }
            let container = self.containers.next()?;
            self.current_key = container.key;
            self.current_values = Some(container.iter());
        }
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        match usize::try_from(self.remaining) {
            Ok(remaining) => (remaining, Some(remaining)),
            Err(_) => (usize::MAX, None),
        }
    }

    fn count(self) -> usize {
        usize::try_from(self.remaining).expect("iterator cardinality exceeds usize")
    }
}

impl FusedIterator for Iter<'_> {}
