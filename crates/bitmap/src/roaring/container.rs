use super::ops::{self, Operation};
use super::store::{ARRAY_LIMIT, ArrayStore, BITMAP_BYTES, RunStore, Store, StoreIter};

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct Container {
    pub(crate) key: u16,
    store: Store,
}

impl Container {
    pub(crate) fn singleton(key: u16, low: u16) -> Self {
        Self {
            key,
            store: Store::singleton(low),
        }
    }

    pub(crate) fn from_range(key: u16, start: u16, end: u16) -> Self {
        let len = u32::from(end) - u32::from(start) + 1;
        let store = if len > 2 {
            Store::Run(RunStore::from_range(start, end))
        } else {
            let mut a = ArrayStore::new();
            a.insert_range(start, end);
            Store::Array(a)
        };
        Self { key, store }
    }
    pub(crate) fn insert_range(&mut self, a: u16, b: u16) -> u32 {
        let n = self.store.insert_range(a, b);
        if n != 0 {
            self.ensure_efficient_store()
        }
        n
    }
    pub(crate) fn remove_range(&mut self, a: u16, b: u16) -> u32 {
        let n = self.store.remove_range(a, b);
        if n != 0 && !self.store.is_empty() {
            self.ensure_efficient_store()
        }
        n
    }
    pub(crate) fn run_optimize(&mut self) -> bool {
        self.store.run_optimize()
    }
    pub(crate) fn combine(&self, other: &Self, op: Operation) -> Self {
        Self {
            key: self.key,
            store: ops::combine(&self.store, &other.store, op),
        }
    }

    pub(crate) fn len(&self) -> u32 {
        self.store.len()
    }

    pub(crate) fn is_empty(&self) -> bool {
        self.store.is_empty()
    }

    pub(crate) fn contains(&self, low: u16) -> bool {
        self.store.contains(low)
    }

    pub(crate) fn iter(&self) -> StoreIter<'_> {
        self.store.iter()
    }

    pub(crate) fn validate(&self) -> bool {
        !self.is_empty() && self.store.validate()
    }

    pub(crate) fn insert(&mut self, low: u16) -> bool {
        let inserted = self.store.insert(low);
        if inserted {
            self.ensure_efficient_store();
        }
        inserted
    }

    pub(crate) fn remove(&mut self, low: u16) -> bool {
        let removed = self.store.remove(low);
        if removed && !self.store.is_empty() {
            self.ensure_efficient_store();
        }
        removed
    }

    fn ensure_efficient_store(&mut self) {
        //this will handle array/bitmap/run conversions.
        enum Conversion {
            None,
            ToArray,
            ToBitmap,
        }
        let conv = match &self.store {
            Store::Array(array) if array.len() > ARRAY_LIMIT => Conversion::ToBitmap,
            Store::Bitmap(bitmap) if bitmap.len() <= ARRAY_LIMIT => Conversion::ToArray,
            Store::Run(run)
                if run.len() <= ARRAY_LIMIT && run.len() as usize * 2 < 2 + run.run_count() * 4 =>
            {
                Conversion::ToArray
            }
            Store::Run(run) if BITMAP_BYTES < 2 + run.run_count() * 4 => Conversion::ToBitmap,
            _ => Conversion::None,
        };
        self.store = match conv {
            Conversion::None => return,
            Conversion::ToBitmap => {
                let current = std::mem::replace(&mut self.store, Store::Array(ArrayStore::new()));
                current.into_bitmap()
            }
            Conversion::ToArray => {
                let current = std::mem::replace(&mut self.store, Store::Array(ArrayStore::new()));
                current.into_array()
            }
        };
    }
}
