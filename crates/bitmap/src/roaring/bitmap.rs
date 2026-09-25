use super::container::Container;
use super::iter::Iter;
use super::ops::Operation;
use super::util::split;

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct RoaringBitmap {
    containers: Vec<Container>,
    cardinality: u64,
}

impl RoaringBitmap {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn len(&self) -> u64 {
        self.cardinality
    }

    pub fn is_empty(&self) -> bool {
        self.cardinality == 0
    }

    pub fn container_count(&self) -> usize {
        self.containers.len()
    }

    pub fn clear(&mut self) {
        self.containers.clear();
        self.cardinality = 0;
    }

    pub fn contains(&self, value: u32) -> bool {
        let (key, low) = split(value);
        let Ok(index) = self
            .containers
            .binary_search_by_key(&key, |container| container.key)
        else {
            return false;
        };
        self.containers[index].contains(low)
    }

    pub fn insert(&mut self, value: u32) -> bool {
        let (key, low) = split(value);
        if self
            .containers
            .last()
            .is_none_or(|container| container.key < key)
        {
            self.containers.push(Container::singleton(key, low));
            self.cardinality += 1;
            return true;
        }

        match self
            .containers
            .binary_search_by_key(&key, |container| container.key)
        {
            Ok(index) => {
                if self.containers[index].insert(low) {
                    self.cardinality += 1;
                    true
                } else {
                    false
                }
            }
            Err(index) => {
                self.containers
                    .insert(index, Container::singleton(key, low));
                self.cardinality += 1;
                true
            }
        }
    }

    pub fn remove(&mut self, value: u32) -> bool {
        let (key, low) = split(value);
        let Ok(index) = self
            .containers
            .binary_search_by_key(&key, |container| container.key)
        else {
            return false;
        };
        if !self.containers[index].remove(low) {
            return false;
        }
        self.cardinality -= 1;
        if self.containers[index].is_empty() {
            self.containers.remove(index);
        }
        true
    }

    pub fn iter(&self) -> Iter<'_> {
        Iter::new(&self.containers, self.cardinality)
    }

    pub fn insert_range(&mut self, range: std::ops::RangeInclusive<u32>) -> u64 {
        if range.is_empty() {
            return 0;
        }
        let start = *range.start();
        let end = *range.end();
        let first = start >> 16;
        let last = end >> 16;
        let mut added = 0;
        for raw_key in first..=last {
            let key = raw_key as u16;
            let low_start = if raw_key == first { start as u16 } else { 0 };
            let low_end = if raw_key == last {
                end as u16
            } else {
                u16::MAX
            };
            match self.containers.binary_search_by_key(&key, |c| c.key) {
                Ok(i) => added += u64::from(self.containers[i].insert_range(low_start, low_end)),
                Err(i) => {
                    let c = Container::from_range(key, low_start, low_end);
                    added += u64::from(c.len());
                    self.containers.insert(i, c)
                }
            }
        }
        self.cardinality += added;
        added
    }

    pub fn remove_range(&mut self, range: std::ops::RangeInclusive<u32>) -> u64 {
        if range.is_empty() {
            return 0;
        }
        let start = *range.start();
        let end = *range.end();
        let first = start >> 16;
        let last = end >> 16;
        let mut removed = 0;
        let mut i = 0;
        while i < self.containers.len() {
            let key = u32::from(self.containers[i].key);
            if key < first {
                i += 1;
                continue;
            }
            if key > last {
                break;
            }
            let a = if key == first { start as u16 } else { 0 };
            let b = if key == last { end as u16 } else { u16::MAX };
            removed += u64::from(self.containers[i].remove_range(a, b));
            if self.containers[i].is_empty() {
                self.containers.remove(i);
            } else {
                i += 1;
            }
        }
        self.cardinality -= removed;
        removed
    }

    pub fn run_optimize(&mut self) -> bool {
        let mut any = false;
        for c in &mut self.containers {
            any |= c.run_optimize()
        }
        any
    }
    pub fn union(&self, other: &Self) -> Self {
        self.combine(other, Operation::Union)
    }
    pub fn intersection(&self, other: &Self) -> Self {
        self.combine(other, Operation::Intersection)
    }
    pub fn difference(&self, other: &Self) -> Self {
        self.combine(other, Operation::Difference)
    }
    pub fn symmetric_difference(&self, other: &Self) -> Self {
        self.combine(other, Operation::SymmetricDifference)
    }

    fn combine(&self, other: &Self, op: Operation) -> Self {
        use std::cmp::Ordering;
        let (mut a, mut b) = (0, 0);
        let mut out = Vec::with_capacity(self.containers.len().max(other.containers.len()));
        while a < self.containers.len() && b < other.containers.len() {
            let (l, r) = (&self.containers[a], &other.containers[b]);
            match l.key.cmp(&r.key) {
                Ordering::Less => {
                    if matches!(
                        op,
                        Operation::Union | Operation::Difference | Operation::SymmetricDifference
                    ) {
                        out.push(l.clone())
                    }
                    a += 1
                }
                Ordering::Greater => {
                    if matches!(op, Operation::Union | Operation::SymmetricDifference) {
                        out.push(r.clone())
                    }
                    b += 1
                }
                Ordering::Equal => {
                    let c = l.combine(r, op);
                    if !c.is_empty() {
                        out.push(c)
                    }
                    a += 1;
                    b += 1
                }
            }
        }
        if matches!(
            op,
            Operation::Union | Operation::Difference | Operation::SymmetricDifference
        ) {
            out.extend_from_slice(&self.containers[a..])
        }
        if matches!(op, Operation::Union | Operation::SymmetricDifference) {
            out.extend_from_slice(&other.containers[b..])
        }
        let cardinality = out.iter().map(|c| u64::from(c.len())).sum();
        Self {
            containers: out,
            cardinality,
        }
    }

    pub(crate) fn validate(&self) -> bool {
        let keys_are_ordered = self
            .containers
            .windows(2)
            .all(|pair| pair[0].key < pair[1].key);
        let containers_are_valid = self.containers.iter().all(Container::validate);

        let calculated_card: u64 = self
            .containers
            .iter()
            .map(|container| u64::from(container.len()))
            .sum();

        keys_are_ordered && containers_are_valid && calculated_card == self.cardinality
    }
}

impl std::ops::BitOr<&RoaringBitmap> for &RoaringBitmap {
    type Output = RoaringBitmap;
    fn bitor(self, rhs: &RoaringBitmap) -> Self::Output {
        self.union(rhs)
    }
}
impl std::ops::BitAnd<&RoaringBitmap> for &RoaringBitmap {
    type Output = RoaringBitmap;
    fn bitand(self, rhs: &RoaringBitmap) -> Self::Output {
        self.intersection(rhs)
    }
}
impl std::ops::Sub<&RoaringBitmap> for &RoaringBitmap {
    type Output = RoaringBitmap;
    fn sub(self, rhs: &RoaringBitmap) -> Self::Output {
        self.difference(rhs)
    }
}
impl std::ops::BitXor<&RoaringBitmap> for &RoaringBitmap {
    type Output = RoaringBitmap;
    fn bitxor(self, rhs: &RoaringBitmap) -> Self::Output {
        self.symmetric_difference(rhs)
    }
}

impl Extend<u32> for RoaringBitmap {
    fn extend<T>(&mut self, values: T)
    where
        T: IntoIterator<Item = u32>,
    {
        for value in values {
            self.insert(value);
        }
    }
}

impl FromIterator<u32> for RoaringBitmap {
    fn from_iter<T>(values: T) -> Self
    where
        T: IntoIterator<Item = u32>,
    {
        let mut bitmap = Self::new();
        bitmap.extend(values);
        bitmap
    }
}

impl<'a> IntoIterator for &'a RoaringBitmap {
    type Item = u32;
    type IntoIter = Iter<'a>;

    fn into_iter(self) -> Self::IntoIter {
        self.iter()
    }
}
