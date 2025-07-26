use std::ops::{Deref, DerefMut};
use lake::{DropletBase, Lake};
use lake::droplet::Droplet;
use lake::lake::droplet::DropletDeserializeExt;

#[test]
fn test_droplet_basic_usage_and_validation() {
    let mut lake: Lake<128> = Lake::<128>::new();
    let mut droplet: Droplet<16, Lake<128>> = lake.alloc::<16>().expect("should allocate droplet");

    assert!(droplet.is_valid());
    assert_eq!(droplet.len(), 16);
    droplet.deref_mut().copy_from_slice(&[1u8; 16]);
    assert_eq!(droplet.deref(), &[1u8; 16]);
}

#[test]
fn test_droplet_as_ptr_and_slice() {
    let mut lake: Lake<64> = Lake::<64>::new();
    let mut droplet: Droplet<8, Lake<64>> = lake.alloc::<8>().unwrap();

    let ptr: *mut u8 = droplet.d_as_mut_ptr();
    unsafe {
        std::ptr::write_bytes(ptr, 42u8, 8);
    }

    assert_eq!(droplet.d_as_mut_slice(), &[42u8; 8]);
    assert_eq!(droplet.d_as_ptr(), ptr as *const u8);
}

#[test]
fn test_droplet_leak_static_access() {
    let mut lake: Lake<64> = Lake::<64>::new();
    let droplet: Droplet<4, Lake<64>> = lake.alloc::<4>().unwrap();

    let leaked: &'static [u8; 4] = unsafe { droplet.leak() };
    assert_eq!(*leaked, [0u8; 4]);
}

#[test]
fn test_droplet_leak_mut_static_access() {
    let mut lake: Lake<64> = Lake::<64>::new();
    let droplet: Droplet<4, Lake<64>> = lake.alloc::<4>().unwrap();

    let leaked: &'static mut [u8; 4] = unsafe { droplet.leak_mut() };
    leaked[0] = 99;
    assert_eq!(leaked[0], 99);
}

#[test]
fn test_droplet_deserialize_struct() {
    #[repr(C)]
    #[derive(Debug, PartialEq, Copy, Clone)]
    struct Point {
        x: u16,
        y: u16,
    }

    let mut lake: Lake<64> = Lake::<64>::new();
    let mut droplet: Droplet<4, Lake<64>> = lake.alloc::<4>().unwrap();
    let buf: &mut [u8; 4] = droplet.deref_mut();
    buf[..2].copy_from_slice(&1u16.to_le_bytes());
    buf[2..4].copy_from_slice(&2u16.to_le_bytes());

    let point: Option<&Point> = droplet.deserialize();
    assert_eq!(point.unwrap(), &Point { x: 1, y: 2 });
}

#[test]
fn test_droplet_deserialize_slice() {
    let mut lake = Lake::<64>::new();
    let mut droplet = lake.alloc::<8>().unwrap();
    droplet.deref_mut().copy_from_slice(&[1u16, 2u16, 3u16, 4u16].iter().flat_map(|v| v.to_le_bytes()).collect::<Vec<_>>());

    let slice: Option<&[u16]> = droplet.deserialize_slice();
    assert_eq!(slice.unwrap(), &[1, 2, 3, 4]);
}

#[test]
fn test_droplet_invalid_after_reset() {
    let mut lake = Lake::<64>::new();
    let droplet = lake.alloc::<8>().unwrap();
    assert!(droplet.is_valid());
    lake.reset(); // generation += 1
    assert!(!droplet.is_valid());
}

#[test]
fn test_droplet_get_lake_mut_and_ptr() {
    let mut lake: Lake<64> = Lake::<64>::new();
    let droplet: Droplet<8, Lake<64>> = lake.alloc::<8>().unwrap();

    unsafe {
        let lake_ptr = droplet.get_lake_ptr();
        let lake_ref: &mut Lake<64> = &mut *lake_ptr;

        let _ = lake_ref.alloc::<4>().unwrap();
        assert_eq!(lake_ref.get_offset(), 8 + 4);
    }
}
