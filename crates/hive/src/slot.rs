pub(crate) struct Slot<T> {
    pub(crate) generation: u32,
    pub(crate) value: Option<T>,
}

impl<T> Slot<T> {
    pub(crate) fn vacant() -> Self {
        Self {
            generation: 0,
            value: None,
        }
    }

    pub(crate) fn matches(&self, generation: u32) -> bool {
        self.generation == generation && self.value.is_some()
    }
}
