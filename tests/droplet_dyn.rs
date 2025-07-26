use lake::{DropletBase, Lake};
use lake::droplet_dyn::DropletDyn;
use lake::lake::droplet::DropletDeserializeExt;

#[test]
fn test_droplet_dyn_process_and_access() {
    let mut lake = Lake::<64>::new();

    let mut dyn_droplet = lake
        .process(|max| vec![0xABu8; max.min(8)])
        .expect("should create droplet");

    assert_eq!(dyn_droplet.d_len(), 8);
    assert_eq!(dyn_droplet.d_as_mut_slice(), &[0xAB; 8]);
    assert!(dyn_droplet.is_valid());
}

#[test]
fn test_droplet_dyn_leak_static() {
    let mut lake = Lake::<64>::new();

    let droplet = lake.process(|_| vec![1, 2, 3]).unwrap();
    let leaked: &'static [u8] = unsafe { droplet.leak() };

    assert_eq!(leaked, &[1, 2, 3]);
}

#[test]
fn test_droplet_dyn_leak_mut_static() {
    let mut lake = Lake::<64>::new();

    let droplet = lake.process(|_| vec![7, 7, 7, 7]).unwrap();
    let leaked_mut: &'static mut [u8] = unsafe { droplet.leak_mut() };

    leaked_mut[2] = 42;
    assert_eq!(leaked_mut, &[7, 7, 42, 7]);
}

#[test]
fn test_droplet_dyn_deserialize_struct() {
    #[repr(C)]
    #[derive(Debug, PartialEq, Copy, Clone)]
    struct Header {
        id: u16,
        code: u16,
    }

    let mut lake = Lake::<64>::new();
    let droplet = lake.process(|_| {
        let mut buf = [0u8; 4];
        buf[..2].copy_from_slice(&1u16.to_le_bytes());
        buf[2..].copy_from_slice(&99u16.to_le_bytes());
        buf.to_vec()
    }).unwrap();

    let header: Option<&Header> = droplet.deserialize();
    assert_eq!(header.unwrap(), &Header { id: 1, code: 99 });
}

#[test]
fn test_droplet_dyn_deserialize_slice() {
    let mut lake = Lake::<64>::new();
    let droplet = lake.process(|_| {
        let nums = [10u16, 20, 30];
        nums.iter().flat_map(|v| v.to_le_bytes()).collect::<Vec<u8>>()
    }).unwrap();

    let slice: Option<&[u16]> = droplet.deserialize_slice();
    assert_eq!(slice.unwrap(), &[10, 20, 30]);
}

#[test]
fn test_droplet_dyn_invalid_after_reset() {
    let mut lake: Lake<64> = Lake::<64>::new();
    let droplet: DropletDyn<64> = lake.process(|_| vec![1, 2, 3, 4]).unwrap();
    assert!(droplet.is_valid());

    lake.reset(); // generation++

    assert!(!droplet.is_valid());
}
