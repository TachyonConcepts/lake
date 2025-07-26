use std::thread;
use std::thread::JoinHandle;
use lake::droplet::Droplet;
use lake::Lake;
use lake::lake::{LakeMeta, LakeSnapshot};

const SIZE: usize = 1024;

fn main() {
    // ✅ Legitimate lake with full borrow-checker support
    let mut lake: Lake<1024> = Lake::<SIZE>::new();

    // 💾 Snapshot before any allocations
    let checkpoint: LakeSnapshot = lake.snapshot();

    println!("Checkpoint: {:?}", checkpoint);

    // 📦 Allocate 64 bytes
    let mut _d1: Droplet<64, Lake<1024>> = lake.alloc::<64>().unwrap();
    println!("After 64B allocation: offset = {}", lake.offset());

    let handler: JoinHandle<()> = thread::spawn(move ||{
        _d1[0] = 123;
    });

    handler.join().unwrap();

    // 💾 Save another snapshot (offset = 64)
    let mid: LakeSnapshot = lake.snapshot();

    // 📦 Allocate another 128 bytes
    let _d2: Droplet<128, Lake<1024>> = lake.alloc::<128>().unwrap();
    println!("After 128B allocation: offset = {}", lake.offset());

    // 🔄 Rewind to mid snapshot (drop _d2 silently)
    lake.rewind(mid);
    println!("After rewind to mid: offset = {}", lake.offset());

    // ⏪ Rewind to original checkpoint (drop _d1 silently)
    lake.rewind(checkpoint);
    println!("After rewind to checkpoint: offset = {}", lake.offset());
}