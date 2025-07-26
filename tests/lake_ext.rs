use lake::{
    lake::LakeAllocatorExt,
    Lake,
};

#[test]
fn test_alloc_struct() {
    #[repr(C)]
    struct Foo {
        x: u32,
        y: u32,
    }

    let mut lake: Lake<128> = Lake::<128>::new();
    let foo: &mut Foo = lake.alloc_struct();
    foo.x = 42;
    foo.y = 99;

    assert_eq!(foo.x, 42);
    assert_eq!(foo.y, 99);
}

#[test]
fn test_alloc_slice() {
    let mut lake: Lake<128> = Lake::<128>::new();
    let slice: &mut [u64] = lake.alloc_slice(3);
    slice.copy_from_slice(&[1, 2, 3]);
    assert_eq!(slice, &[1, 2, 3]);
}
