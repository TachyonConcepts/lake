use std::hint::black_box;
use lake::droplet::Droplet;
use lake::Lake;
use lake::lake::LakeSnapshot;

fn main() {
    // === BLOCK 1: Basic mark and rollback ===
    let mut lake: Lake<1024> = Lake::<1024>::new();
    // Save the current allocation offset (0) into the mark stack.
    lake.mark();
    // Allocate 64 bytes → offset becomes 64.
    let _a: Droplet<64, Lake<1024>> = lake.alloc::<64>().unwrap();
    // Allocate 128 bytes → offset becomes 64 + 128 = 192.
    let _b: Droplet<128, Lake<1024>> = lake.alloc::<128>().unwrap();
    assert_eq!(lake.get_offset(), 192);
    // Roll back to the last mark (which was 0) → offset reset to 0.
    lake.reset_to_mark();
    assert_eq!(lake.get_offset(), 0);

    // === BLOCK 2: Nested marks and sequential rollback ===
    let mut lake: Lake<1024> = Lake::<1024>::new();
    // Mark current position (offset = 0).
    lake.mark();
    // Allocate 100 bytes → offset = 100.
    let _ = lake.alloc::<100>().unwrap();
    // Mark again at offset 100 (nested mark).
    lake.mark();
    // Allocate 200 bytes → offset = 300.
    let _ = lake.alloc::<200>().unwrap();
    // Roll back to the last mark (offset = 100).
    lake.reset_to_mark();
    assert_eq!(lake.get_offset(), 100);
    // Roll back to the first mark (offset = 0).
    lake.reset_to_mark();
    assert_eq!(lake.get_offset(), 0);

    // === BLOCK 3: Moving the last mark to current offset ===
    let mut lake: Lake<1024> = Lake::<1024>::new();
    // Save offset 0 into the mark stack.
    lake.mark();
    // Allocate 64 bytes → offset = 64.
    let _ = lake.alloc::<64>().unwrap();
    // Update the most recent mark to point to the current offset (64 instead of 0).
    lake.move_mark();
    // Allocate another 64 bytes → offset = 128.
    let _ = lake.alloc::<64>().unwrap();
    // Roll back to updated mark (64) instead of original mark (0).
    lake.reset_to_mark();
    assert_eq!(lake.get_offset(), 64);

    // === BLOCK 4: Safe rollback after droplet usage ===
    let mut lake: Lake<1024> = Lake::<1024>::new();
    // Save offset = 0
    lake.mark();
    // Allocate two droplets (128 + 128) → offset = 256.
    let d1: Droplet<128, Lake<1024>> = lake.alloc::<128>().unwrap();
    let d2: Droplet<128, Lake<1024>> = lake.alloc::<128>().unwrap();
    // Use the droplets (read-only) without changing the allocator state.
    black_box((d1.as_ref(), d2.as_ref()));
    // Roll back to the original mark (offset = 0).
    lake.reset_to_mark();
    // After rollback, lake offset should be back to < 128 (should be exactly 0).
    assert!(lake.get_offset() < 128);

    // === BLOCK 5: Snapshot-based rewind and mark stack consistency ===
    let mut lake: Lake<1024> = Lake::<1024>::new();
    // Take a snapshot of the current offset (0). This is a value-based checkpoint.
    let snap: LakeSnapshot = lake.snapshot();
    // Also push a stack-based mark.
    lake.mark();
    // Allocate 100 + 200 bytes → offset = 300.
    let _ = lake.alloc::<100>().unwrap();
    let _ = lake.alloc::<200>().unwrap();
    // Rewind using snapshot → offset = 0. Mark stack still contains one entry.
    lake.rewind(snap);
    assert_eq!(lake.get_offset(), 0);
    // Pop the mark (which was still at offset 0) — no change expected.
    lake.reset_to_mark();
    assert_eq!(lake.get_offset(), 0);
}