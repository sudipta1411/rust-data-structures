#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Handle {
    pub(crate) block: u32,
    pub(crate) slot: u32,
    pub(crate) generation: u64,
}

impl Handle {
    pub fn block(self) -> usize {
        self.block as usize
    }

    pub fn slot(self) -> usize {
        self.slot as usize
    }

    pub fn generation(self) -> u64 {
        self.generation
    }
}
