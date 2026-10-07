use super::{Handle, Hive, Location};

pub struct VacantEntry<'a, T> {
    hive: &'a mut Hive<T>,
    location: Option<Location>,
}

impl<'a, T> VacantEntry<'a, T> {
    pub(crate) fn new(hive: &'a mut Hive<T>) -> Self {
        let location = hive.reserve_location();
        Self {
            hive,
            location: Some(location),
        }
    }

    pub fn insert(mut self, value: T) -> Handle {
        let location = self.location.expect("vacant entry has no location");
        let handle = self.hive.insert_at(location, value);
        self.location = None;
        handle
    }
}

impl<T> Drop for VacantEntry<'_, T> {
    fn drop(&mut self) {
        if let Some(location) = self.location.take() {
            self.hive.free.push(location);
        }
    }
}
