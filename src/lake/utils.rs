#[inline(always)]
pub fn align_up(offset: usize, align: usize) -> usize {
    assert!(align.is_power_of_two());
    (offset + align - 1) & !(align - 1)
}