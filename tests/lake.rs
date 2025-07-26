use lake::droplet::Droplet;
use lake::Lake;
use lake::lake::LakeSnapshot;

#[test]
fn test_new_lake_is_empty() {
    let lake: Lake<128> = Lake::<128>::new();
    assert_eq!(lake.used(), 0);
    assert_eq!(lake.remaining(), 128);
    assert!(lake.is_empty());
    assert!(!lake.is_full());
}

#[test]
fn test_alloc_fixed_droplet() {
    let mut lake: Lake<64> = Lake::<64>::new();
    let droplet: Droplet<16, Lake<64>> = lake.alloc::<16>().expect("Should allocate");
    assert_eq!(lake.used(), 16);
    assert_eq!(lake.remaining(), 48);
    assert_eq!(droplet.as_ref(), &[0u8; 16]);
}

#[test]
fn test_alloc_overflow() {
    let mut lake = Lake::<16>::new();
    assert!(lake.alloc::<32>().is_none());
}

#[test]
fn test_reset_behavior() {
    let mut lake: Lake<64> = Lake::<64>::new();
    let _ = lake.alloc::<32>().unwrap();
    lake.set_zeroing(true);
    assert_eq!(lake.as_slice(), &[0u8; 32]);
    lake.as_mut_slice().copy_from_slice(&[1u8; 32]);
    lake.reset();
    assert_eq!(lake.as_slice(), &[0u8; 0]);
    assert_eq!(lake.used(), 0);
}

#[test]
fn test_mark_and_reset_to_mark() {
    let mut lake: Lake<64> = Lake::<64>::new();
    let _ = lake.alloc::<10>().unwrap();
    lake.mark();
    let _ = lake.alloc::<20>().unwrap();
    lake.reset_to_mark();
    assert_eq!(lake.used(), 10);
}

#[test]
fn test_move_mark() {
    let mut lake: Lake<64> = Lake::<64>::new();
    let _ = lake.alloc::<8>().unwrap();
    lake.mark(); // offset = 8
    let _ = lake.alloc::<16>().unwrap(); // offset = 24
    lake.move_mark();
    lake.reset_to_mark(); // should reset to 24
    assert_eq!(lake.used(), 24);
}

#[test]
fn test_snapshot_and_rewind() {
    let mut lake: Lake<64> = Lake::<64>::new();
    let _ = lake.alloc::<16>().unwrap();
    let snap: LakeSnapshot = lake.snapshot();
    let _ = lake.alloc::<16>().unwrap();
    lake.rewind(snap);
    assert_eq!(lake.used(), 16);
}

#[test]
fn test_as_slice_and_mut_slice() {
    let mut lake: Lake<32> = Lake::<32>::new();
    let slice: &mut [u8] = lake.as_mut_slice();
    assert_eq!(slice.len(), 0);
    let _ = lake.alloc::<8>().unwrap();
    lake.as_mut_slice()[0] = 42;
    assert_eq!(lake.as_slice()[0], 42);
}

#[test]
fn test_peek_behavior() {
    let mut lake: Lake<32> = Lake::<32>::new();
    let _ = lake.alloc::<8>().unwrap();
    let peek = lake.peek::<8>().unwrap();
    assert_eq!(peek, &[0u8; 8]);
}

#[test]
fn test_reset_to_bytes() {
    let mut lake: Lake<64> = Lake::<64>::new();
    let _ = lake.alloc::<32>().unwrap();
    lake.reset_to(16);
    assert_eq!(lake.used(), 16);
}
