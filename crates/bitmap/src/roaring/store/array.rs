#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub(crate) struct ArrayStore {
    values: Vec<u16>,
}

impl ArrayStore {
    pub(crate) fn new() -> Self {
        Self::default()
    }

    pub(crate) fn singleton(value: u16) -> Self {
        Self {
            values: vec![value],
        }
    }

    pub(crate) fn from_sorted_values(values: Vec<u16>) -> Self {
        debug_assert!(
            values.windows(2).all(|pair| pair[0] < pair[1]),
            "array values must be strictly increasing",
        );
        Self { values }
    }

    pub(crate) fn into_values(self) -> Vec<u16> {
        self.values
    }

    pub(crate) fn values(&self) -> &[u16] {
        &self.values
    }

    pub(crate) fn len(&self) -> u32 {
        self.values
            .len()
            .try_into()
            .expect("an array container cannot exceed 65,536 values")
    }

    pub(crate) fn contains(&self, value: u16) -> bool {
        self.values.binary_search(&value).is_ok()
    }

    pub(crate) fn insert(&mut self, value: u16) -> bool {
        match self.values.binary_search(&value) {
            Ok(_) => false,
            Err(index) => {
                self.values.insert(index, value);
                true
            }
        }
    }

    pub(crate) fn remove(&mut self, value: u16) -> bool {
        match self.values.binary_search(&value) {
            Ok(index) => {
                self.values.remove(index);
                true
            }
            Err(_) => false,
        }
    }

    pub(crate) fn insert_range(&mut self, start: u16, end: u16) -> u32 {
        assert!(start <= end, "range start must not exceed range end");

        let first = self.values.partition_point(|&v| v < start);
        let after = self.values.partition_point(|&v| v <= end);
        let added = u32::from(end) - u32::from(start) + 1 - (after - first) as u32;
        if added != 0 {
            let mut values = Vec::with_capacity(self.values.len() + added as usize);
            values.extend_from_slice(&self.values[..first]);
            values.extend(start..=end);
            values.extend_from_slice(&self.values[after..]);
            self.values = values;
        }
        added
    }

    pub(crate) fn remove_range(&mut self, start: u16, end: u16) -> u32 {
        assert!(start <= end, "range start must not exceed range end");

        let first = self.values.partition_point(|&v| v < start);
        let after = self.values.partition_point(|&v| v <= end);
        let removed = (after - first) as u32;
        self.values.drain(first..after);
        removed
    }

    pub(crate) fn iter(&self) -> std::slice::Iter<'_, u16> {
        self.values.iter()
    }

    pub(crate) fn validate(&self) -> bool {
        self.values.windows(2).all(|pair| pair[0] < pair[1])
    }
}
