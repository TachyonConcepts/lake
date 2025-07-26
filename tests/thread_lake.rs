use std::panic;
use lake::thread_lake::{thread_lake_init, DEFAULT_SIZE};
use lake::{with_lake, Lake};

#[test]
fn test_thread_lake_initialization_and_access() {
    thread_lake_init();
    with_lake!(|lake: &mut Lake<DEFAULT_SIZE>| {
        assert_eq!(lake.capacity(), DEFAULT_SIZE);
        assert_eq!(lake.used(), 0);
        let _ = lake.alloc::<8>().unwrap();
        assert_eq!(lake.used(), 8);
    });
}

#[test]
fn test_thread_lake_reuse_in_same_thread() {
    thread_lake_init();
    with_lake!(|lake: &mut Lake<DEFAULT_SIZE>| {
        let _ = lake.alloc::<16>().unwrap();
    });
    with_lake!(|lake: &mut Lake<DEFAULT_SIZE>| {
        assert!(lake.used() >= 16);
    });
}

#[test]
fn test_thread_lake_panic_without_init() {
    lake::thread_lake::THREAD_LAKE.with(|slot| {
        *slot.borrow_mut() = None;
    });

    let result: std::thread::Result<()> = panic::catch_unwind(|| {
        with_lake!(|_lake| {
            // ...
        });
    });

    assert!(result.is_err());
    if let Err(err) = result {
        let msg: Option<&str> = err
            .downcast_ref::<&'static str>()
            .copied()
            .or_else(|| err.downcast_ref::<String>().map(|s| s.as_str()));
        assert!(msg.unwrap_or("").contains("Lake not initialized"));
    }
}
