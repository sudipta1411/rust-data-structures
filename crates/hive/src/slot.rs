pub(crate) struct Slot<T> {
    pub(crate) generation: u64,
    pub(crate) value: Option<T>,
}

impl<T> Slot<T> {
    pub(crate) fn vacant() -> Self {
        Self {
            generation: 0,
            value: None,
        }
    }

    pub(crate) fn matches(&self, generation: u64) -> bool {
        self.generation == generation && self.value.is_some()
    }

    pub(crate) fn generation_matches(&self, generation: u64) -> bool {
        self.generation == generation
    }
}
