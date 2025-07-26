use lake::{
    droplet::Droplet, lake::{memory::void::Void, LakeMeta}, thread_lake::{thread_lake_init, DEFAULT_SIZE},
    with_lake,
    DropletBase,
    Lake,
    FBC,
};

const SIZE: usize = 1024;

fn main() {
    // ✅ Method 1: Create a lake manually
    let lake: Lake<1024> = Lake::<SIZE>::new();
    assert_eq!(lake.offset(), 0);
    println!("Manual lake created with size: {}", SIZE);

    // ✅ Method 2: Use thread-local lake via macro
    thread_lake_init(); // Initialize the lake for this thread
    with_lake!(|lake: &mut Lake<DEFAULT_SIZE>| {
        println!("Thread-local lake ready with size: {}", DEFAULT_SIZE);
        assert_eq!(lake.offset(), 0);
    });

    // ✅ Method 3: Use Box to heap-allocate a lake
    let boxed: Box<Lake<1024>> = Box::new(Lake::<SIZE>::new());
    assert_eq!(boxed.capacity(), SIZE);
    println!("Boxed lake created with size: {}", boxed.capacity());

    // ⚠️ Method 4: Use FBC! macro to leak a lake as a static mutable reference
    // FBC! with lake is **not** recommended for production. Safe to use with droplets only.

    let result: std::thread::Result<()> = std::panic::catch_unwind(|| {
        let mut static_lake: Void<Lake<1024>> = FBC!(Lake::<SIZE>::new());
        static_lake.mark();
        assert_eq!(static_lake.offset(), 0);

        let mut droplet: Droplet<16, Lake<1024>> =
            unsafe { (*static_lake.0).alloc::<16>().unwrap() };
        assert_eq!(static_lake.offset(), 16);

        static_lake.reset_to_mark(); // reset lake to mark
        assert_eq!(static_lake.offset(), 0);
        droplet.d_as_mut_slice(); // now this is outlived droplet and this method should panic
        println!("Leaked lake with FBC! was used and rewound correctly");
    });
    match result {
        Ok(_) => {
            eprintln!("❌ Expected panic, but code ran successfully!");
            std::process::exit(1);
        }
        Err(_) => {
            println!("✅ Panic occurred as expected.");
        }
    }
}
