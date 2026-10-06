#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Handle {
    pub(crate) block: u32,
    pub(crate) slot: u32,
    pub(crate) generation: u64,
}

impl Handle {
    pub(crate) fn new(block: usize, slot: usize, generation: u64) -> Self {
        Self {
            block: u32::try_from(block).expect("block index exceeds u32::MAX"),
            slot: u32::try_from(slot).expect("slot index exceeds u32::MAX"),
            generation,
        }
    }

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
