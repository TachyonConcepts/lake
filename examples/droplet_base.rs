use lake::droplet::Droplet;
use lake::Lake;
use lake::lake::LakeMeta;

fn example_droplet_validity() {
    let mut lake: Lake<1024> = Lake::<1024>::new();
    let droplet: Droplet<128, Lake<1024>> = lake.alloc::<128>().unwrap();
    // Check whether the droplet is still considered valid
    assert!(droplet.is_valid());
    // Manually rewind lake beyond droplet
    lake.reset();
    // Now the droplet is stale (invalid)
    assert!(!droplet.is_valid());
}

fn example_leak_static_reference() {
    let mut lake: Lake<1024> = Lake::<1024>::new();
    let droplet: Droplet<64, Lake<1024>> = lake.alloc::<64>().unwrap();
    // Leak the buffer to 'static. You MUST ensure that the lake lives forever!
    let leaked: &'static [u8; 64] = unsafe { droplet.leak() };
    leaked[0]; // ✅ works
    // do NOT use after lake is dropped
}

fn example_leak_mut_static() {
    let mut lake: Lake<1024> = Lake::<1024>::new();
    let droplet: Droplet<16, Lake<1024>> = lake.alloc::<16>().unwrap();
    let leaked_mut: &'static mut [u8; 16] = unsafe { droplet.leak_mut() };
    leaked_mut[0] = 42;
    leaked_mut[15] = 99;
}

fn example_get_lake_ptr() {
    let mut lake: Lake<1024> = Lake::<1024>::new();
    let droplet: Droplet<64, Lake<1024>> = lake.alloc::<64>().unwrap();
    let lake_ptr: *mut Lake<1024> = droplet.get_lake_ptr();
    // Can be used for advanced pointer-level operations
    unsafe {
        (*lake_ptr).reset(); // Directly reset the lake from the pointer
    }
    assert!(!droplet.is_valid()); // Now droplet is invalid
}

fn example_mutate_lake_from_droplet() {
    let mut lake: Lake<1024> = Lake::<1024>::new();
    let mut droplet: Droplet<32, Lake<1024>> = lake.alloc::<32>().unwrap();
    unsafe {
        let lake_ref: &mut dyn LakeMeta = droplet.get_lake_mut();
        lake_ref.offset_mut().clone_from(&mut 0); // Manual offset reset
    }
    assert!(!droplet.is_valid());
}

fn main() {
    example_leak_mut_static();
    example_leak_static_reference();
    example_droplet_validity();
    example_get_lake_ptr();
    example_mutate_lake_from_droplet();
}