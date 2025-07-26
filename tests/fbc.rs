use lake::FBC;
use lake::lake::memory::void::Void;

#[test]
fn test_void_deref_and_mut() {
    let mut value: i32 = 123;
    let mut ptr = Void(&mut value as *mut i32);
    assert_eq!(*ptr, 123);

    *ptr = 456;
    assert_eq!(unsafe { *ptr.0 }, 456);
}

#[test]
fn test_void_copy_and_clone() {
    let mut value: i32 = 77;
    let a = Void(&mut value as *mut i32);
    let b: Void<i32> = a;
    let c: i32 = *a.clone();

    assert_eq!(*a, 77);
    assert_eq!(*b, 77);
    assert_eq!(c, 77);
}

#[test]
fn test_fbc_macro_leaked_value_is_accessible() {
    let mut wrapped: Void<Vec<u8>> = FBC!(vec![1, 2, 3]);
    assert_eq!(wrapped.len(), 3);
    assert_eq!(wrapped[1], 2);

    wrapped[1] = 42;
    assert_eq!(wrapped[1], 42);
}

#[test]
#[should_panic]
fn test_fbc_panics_under_miri_simulated() {
    if cfg!(miri) {
        let _ = FBC!(vec![1, 2, 3]);
    } else {
        panic!("miri only");
    }
}
