#[inline]
pub(crate) const fn split(value: u32) -> (u16, u16) {
    ((value >> 16) as u16, value as u16)
}

#[inline]
pub(crate) const fn join(key: u16, low: u16) -> u32 {
    ((key as u32) << 16) | low as u32
}
