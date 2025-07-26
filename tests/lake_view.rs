use lake::droplet::Droplet;
use lake::droplet_dyn::DropletDyn;
use lake::lake_view::LakeView;
use lake::Lake;
use lake::lake::{LakeAllocatorExt, LakeError};

#[test]
fn test_lake_view_new_alloc_and_usage() {
    let mut lake: Lake<64> = Lake::<64>::new();
    let mut view: LakeView<64> = lake.split(32).expect("should split view");

    assert_eq!(view.used(), 0);
    assert_eq!(view.remaining(), 32);
    assert_eq!(view.capacity(), 32);

    let droplet: Droplet<16, LakeView<64>> = view.alloc::<16>().expect("should alloc");
    assert_eq!(view.used(), 16);
    assert_eq!(view.remaining(), 16);
    assert_eq!(&*droplet, &[0u8; 16]);
}

#[test]
fn test_lake_view_alloc_overflow() {
    let mut bind: [u8; 16] = [0u8; 16];
    let mut view: LakeView<16> = LakeView::<16>::new(&mut bind);
    let _ = view.alloc::<8>().unwrap();
    assert!(view.alloc::<9>().is_none());
}

#[test]
fn test_lake_view_reset_zeroing() {
    let mut buffer: [u8; 32] = [1u8; 32];
    let mut view: LakeView<32> = LakeView::<32>::new(&mut buffer);
    view.set_zeroing(true);
    let _ = view.alloc::<16>().unwrap();
    view.clear();
    let slice: &[u8] = unsafe { std::slice::from_raw_parts(view.buf, 32) };
    assert!(slice[..16].iter().all(|&b| b == 0));
}

#[test]
fn test_lake_view_mark_reset() {
    let mut bind: [u8; 32] = [0u8; 32];
    let mut view: LakeView<32> = LakeView::<32>::new(&mut bind);
    let _ = view.alloc::<8>().unwrap();
    view.mark();
    let _ = view.alloc::<8>().unwrap();
    view.reset_to_mark();
    assert_eq!(view.used(), 8);
}

#[test]
fn test_lake_view_move_mark() {
    let mut bind: [u8; 32] = [0u8; 32];
    let mut view: LakeView<32> = LakeView::<32>::new(&mut bind);
    let _ = view.alloc::<4>().unwrap(); // offset = 4
    view.mark(); // push 4
    let _ = view.alloc::<6>().unwrap(); // offset = 10
    view.move_mark(); // mark should now be 10
    let _ = view.alloc::<2>().unwrap(); // offset = 12
    view.reset_to_mark(); // back to 10
    assert_eq!(view.used(), 10);
}

#[test]
fn test_lake_view_split_subview() {
    let mut bind: [u8; 32] = [0u8; 32];
    let mut view: LakeView<32> = LakeView::<32>::new(&mut bind);
    let _ = view.alloc::<8>().unwrap(); // offset = 8
    let sub: LakeView<32> = view.split(16).expect("split");
    assert_eq!(sub.capacity(), 16);
    assert_eq!(view.used(), 24);
}

#[test]
fn test_lake_view_process_success() {
    let mut bind: [u8; 64] = [0u8; 64];
    let mut view: LakeView<64> = LakeView::<64>::new(&mut bind);
    let result: Result<DropletDyn<64>, LakeError> = view.process(|remaining| vec![42u8; remaining.min(8)]);
    assert!(result.is_ok());
    let droplet: DropletDyn<64> = result.unwrap();
    let slice: &[u8] = unsafe { std::slice::from_raw_parts(droplet.ptr.as_ptr(), droplet.len) };
    assert_eq!(slice, &[42u8; 8]);
    assert_eq!(view.used(), 8);
}

#[test]
fn test_lake_view_process_overflow() {
    let mut bind: [u8; 16] = [0u8; 16];
    let mut view: LakeView<16> = LakeView::<16>::new(&mut bind);
    let result: Result<DropletDyn<16>, LakeError> = view.process(|_| vec![1u8; 32]); // too much
    assert!(matches!(result, Err(LakeError::Overflow)));
}

#[test]
fn test_lake_view_alloc_struct() {
    #[repr(C)]
    struct Foo {
        a: u32,
        b: u32,
    }
    let mut bind: [u8; 64] = [0u8; 64];
    let mut view: LakeView<64> = LakeView::<64>::new(&mut bind);
    let foo: &mut Foo = view.alloc_struct::<Foo>();
    foo.a = 123;
    foo.b = 456;
    assert_eq!(foo.a, 123);
    assert_eq!(foo.b, 456);
}

#[test]
fn test_lake_view_alloc_slice() {
    let mut bind: [u8; 64] = [0u8; 64];
    let mut view: LakeView<64> = LakeView::<64>::new(&mut bind);
    let slice: &mut [u16] = view.alloc_slice(4);
    slice.copy_from_slice(&[1, 2, 3, 4]);
    assert_eq!(slice, &[1, 2, 3, 4]);
}
