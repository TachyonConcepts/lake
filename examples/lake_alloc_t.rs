/*⚠️ Disclaimer: Use With Caution
The Lake allocator is primarily designed to operate on raw u8 memory slices for high-performance buffer management.
While it exposes methods like alloc_struct<T>() and alloc_slice<T>() for convenience,
these are inherently unsafe and rely on manual alignment and layout assumptions.
Using non-u8 types — especially those with drop logic, references,
or invalid memory representations — is not recommended unless you fully understand the implications.
These methods are provided for advanced, low-level use cases such as POD structs, byte-packed views,
or transmuted primitives.
*/
use lake::lake::LakeAllocatorExt;
use lake::Lake;

fn example_1() {
    let mut lake: Lake<1024> = Lake::new();
    // Allocate one u32 from the lake.
    // Internally aligns the offset to 4 bytes (alignment of u32).
    let value: &mut u32 = lake.alloc_struct::<u32>();
    *value = 12345;
    assert_eq!(*value, 12345);
}

fn example_2() {
    #[repr(C)]
    struct Header {
        magic: u16,
        version: u16,
        length: u32,
    }
    let mut lake: Lake<1024> = Lake::new();
    // Allocate a Header struct.
    // Lake will align to 4 bytes (since u32 is the largest field).
    let header: &mut Header = lake.alloc_struct::<Header>();
    header.magic = 0xABCD;
    header.version = 1;
    header.length = 4096;
    assert_eq!(header.length, 4096);
}

fn example_3() {
    let mut lake: Lake<1024> = Lake::new();
    // Allocate a slice of 32 u16s → 64 bytes, aligned to 2.
    let array: &mut [u16] = lake.alloc_slice::<u16>(32);
    for (i, item) in array.iter_mut().enumerate() {
        *item = i as u16;
    }
    assert_eq!(array[0], 0);
    assert_eq!(array[31], 31);
}

fn example_4() {
    struct Bad {
        _marker: std::marker::PhantomData<&'static str>,
    }
    // ❌ Avoid this — types with zero size or internal references can cause UB.
    let mut lake: Lake<1024> = Lake::new();
    let _ = lake.alloc_struct::<Bad>(); // UB or meaningless
}

fn example_5() {
    #[repr(C, align(16))]
    struct Wide {
        data: [u8; 48],
    }
    let mut lake: Lake<1024> = Lake::new();
    // Allocate a 16-byte aligned struct.
    let ptr: *const Wide = lake.alloc_struct::<Wide>() as *const Wide;
    // Verify alignment manually.
    assert_eq!((ptr as usize) % 16, 0);
}

fn main() {
    example_1();
    example_2();
    example_3();
    example_4();
    example_5();
}
