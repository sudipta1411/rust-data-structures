mod array;
mod bitmap;
mod run;

pub(crate) use array::ArrayStore;
pub(crate) use bitmap::{BitmapIter, BitmapStore};
pub(crate) use run::{Run, RunIter, RunStore};

pub(crate) const ARRAY_LIMIT: u32 = 4096;
pub(crate) const CONTAINER_BITS: usize = 1 << 16;
pub(crate) const WORD_BITS: usize = u64::BITS as usize;
pub(crate) const BITMAP_WORDS: usize = CONTAINER_BITS / WORD_BITS;
pub(crate) const BITMAP_BYTES: usize = BITMAP_WORDS * size_of::<u64>();

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum Store {
    Array(ArrayStore),
    Bitmap(BitmapStore),
    Run(RunStore),
}

impl Store {
    pub(crate) fn singleton(v: u16) -> Self {
        Self::Array(ArrayStore::singleton(v))
    }
    pub(crate) fn len(&self) -> u32 {
        match self {
            Self::Array(s) => s.len(),
            Self::Bitmap(s) => s.len(),
            Self::Run(s) => s.len(),
        }
    }
    pub(crate) fn is_empty(&self) -> bool {
        self.len() == 0
    }
    pub(crate) fn contains(&self, v: u16) -> bool {
        match self {
            Self::Array(s) => s.contains(v),
            Self::Bitmap(s) => s.contains(v),
            Self::Run(s) => s.contains(v),
        }
    }
    pub(crate) fn insert(&mut self, v: u16) -> bool {
        match self {
            Self::Array(s) => s.insert(v),
            Self::Bitmap(s) => s.insert(v),
            Self::Run(s) => s.insert(v),
        }
    }
    pub(crate) fn remove(&mut self, v: u16) -> bool {
        match self {
            Self::Array(s) => s.remove(v),
            Self::Bitmap(s) => s.remove(v),
            Self::Run(s) => s.remove(v),
        }
    }
    pub(crate) fn insert_range(&mut self, a: u16, b: u16) -> u32 {
        match self {
            Self::Array(s) => s.insert_range(a, b),
            Self::Bitmap(s) => s.insert_range(a, b),
            Self::Run(s) => s.insert_range(a, b),
        }
    }
    pub(crate) fn remove_range(&mut self, a: u16, b: u16) -> u32 {
        match self {
            Self::Array(s) => s.remove_range(a, b),
            Self::Bitmap(s) => s.remove_range(a, b),
            Self::Run(s) => s.remove_range(a, b),
        }
    }
    pub(crate) fn iter(&self) -> StoreIter<'_> {
        match self {
            Self::Array(s) => StoreIter::Array(s.iter()),
            Self::Bitmap(s) => StoreIter::Bitmap(s.iter()),
            Self::Run(s) => StoreIter::Run(s.iter()),
        }
    }
    pub(crate) fn into_bitmap_store(self) -> BitmapStore {
        match self {
            Self::Array(s) => BitmapStore::from_array(s),
            Self::Bitmap(s) => s,
            Self::Run(s) => {
                let mut b = BitmapStore::new();
                for r in s.runs() {
                    b.insert_range(r.start, r.end);
                }
                b
            }
        }
    }
    pub(crate) fn into_bitmap(self) -> Self {
        Self::Bitmap(self.into_bitmap_store())
    }
    pub(crate) fn into_array(self) -> Self {
        match self {
            Self::Array(s) => Self::Array(s),
            Self::Bitmap(s) => Self::Array(s.into_array()),
            Self::Run(s) => Self::Array(ArrayStore::from_sorted_values(s.iter().collect())),
        }
    }
    pub(crate) fn run_optimize(&mut self) -> bool {
        let runs = values_to_runs(self.iter());
        let bytes = 2 + runs.len() * 4;
        let current = match self {
            Self::Array(s) => s.len() as usize * 2,
            Self::Bitmap(_) => BITMAP_BYTES,
            Self::Run(_) => return true,
        };
        if bytes < current {
            *self = Self::Run(RunStore::from_runs(runs));
            true
        } else {
            false
        }
    }
    pub(crate) fn validate(&self) -> bool {
        match self {
            Self::Array(s) => s.len() <= ARRAY_LIMIT && s.validate(),
            Self::Bitmap(s) => s.len() > ARRAY_LIMIT && s.validate(),
            Self::Run(s) => s.validate(),
        }
    }
}

pub(crate) fn values_to_runs<I: IntoIterator<Item = u16>>(values: I) -> Vec<Run> {
    let mut i = values.into_iter();
    let Some(first) = i.next() else { return vec![] };
    let (mut a, mut b) = (first, first);
    let mut out = Vec::new();
    for v in i {
        if u32::from(b) + 1 == u32::from(v) {
            b = v
        } else {
            out.push(Run::new(a, b));
            a = v;
            b = v
        }
    }
    out.push(Run::new(a, b));
    out
}

pub(crate) enum StoreIter<'a> {
    Array(std::slice::Iter<'a, u16>),
    Bitmap(BitmapIter<'a>),
    Run(RunIter<'a>),
}
impl Iterator for StoreIter<'_> {
    type Item = u16;
    fn next(&mut self) -> Option<u16> {
        match self {
            Self::Array(i) => i.next().copied(),
            Self::Bitmap(i) => i.next(),
            Self::Run(i) => i.next(),
        }
    }
    fn size_hint(&self) -> (usize, Option<usize>) {
        match self {
            Self::Array(i) => i.size_hint(),
            Self::Bitmap(i) => i.size_hint(),
            Self::Run(i) => i.size_hint(),
        }
    }
}
impl ExactSizeIterator for StoreIter<'_> {
    fn len(&self) -> usize {
        match self {
            Self::Array(i) => i.len(),
            Self::Bitmap(i) => i.len(),
            Self::Run(i) => i.len(),
        }
    }
}
impl std::iter::FusedIterator for StoreIter<'_> {}
