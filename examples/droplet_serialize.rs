use lake::droplet::Droplet;
use lake::lake::droplet::DropletDeserializeExt;
use lake::Lake;

#[repr(C)]
#[derive(Debug, PartialEq, Copy, Clone)]
struct Header {
    magic: u16,
    version: u16,
    flags: u16,
}

fn example_deserialize_struct() {
    let mut lake: Lake<1024> = Lake::<1024>::new();
    let mut droplet = lake.alloc::<{ size_of::<Header>() }>().unwrap();
    let raw: &mut [u8] = droplet.as_mut();
    // Write 3 fields as little-endian bytes
    raw[0..2].copy_from_slice(&0xABCDu16.to_le_bytes()); // magic
    raw[2..4].copy_from_slice(&0x0100u16.to_le_bytes()); // version
    raw[4..6].copy_from_slice(&0xFF00u16.to_le_bytes()); // flags
    let header: &Header = droplet.deserialize::<Header>().unwrap();
    assert_eq!(
        *header,
        Header {
            magic: 0xABCD,
            version: 0x0100,
            flags: 0xFF00,
        }
    );
}

fn example_deserialize_slice() {
    let mut lake: Lake<64> = Lake::<64>::new();
    let mut droplet: Droplet<16, Lake<64>> = lake.alloc::<16>().unwrap(); // 4 x u32
    let raw: &mut [u8] = droplet.as_mut();
    let numbers: [u32; 4] = [1, 2, 3, 4];
    let bytes: &[u8] = unsafe {
        core::slice::from_raw_parts(
            numbers.as_ptr() as *const u8,
            numbers.len() * size_of::<u32>(),
        )
    };
    raw.copy_from_slice(bytes);
    let array: &[u32] = droplet.deserialize_slice::<u32>().unwrap();
    assert_eq!(array, &[1, 2, 3, 4]);
}

fn example_deserialize_failure() {
    let mut lake: Lake<64> = Lake::<64>::new();
    // Allocate 6 bytes only, but Header is 8 bytes
    let droplet: Droplet<5, Lake<64>> = lake.alloc::<5>().unwrap();
    // This should fail: struct does not fit
    assert!(droplet.deserialize::<Header>().is_none());
    // This also fails: slice size is not multiple of u32
    assert!(droplet.deserialize_slice::<u32>().is_none());
}

fn main() {
    example_deserialize_struct();
    example_deserialize_slice();
    example_deserialize_failure();
}
