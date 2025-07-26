#[cfg(test)]
mod tests {
    use lake::utils::align_up;

    #[test]
    fn test_align_up_already_aligned() {
        assert_eq!(align_up(64, 8), 64);
        assert_eq!(align_up(128, 16), 128);
    }

    #[test]
    fn test_align_up_needs_alignment() {
        assert_eq!(align_up(65, 8), 72);
        assert_eq!(align_up(130, 16), 144);
    }

    #[test]
    fn test_align_up_zero_offset() {
        assert_eq!(align_up(0, 8), 0);
        assert_eq!(align_up(0, 16), 0);
    }

    #[test]
    #[should_panic]
    fn test_align_up_non_power_of_two() {
        align_up(5, 3);
    }
}