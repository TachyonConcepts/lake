use lake::droplet::Droplet;
use lake::droplet_dyn::DropletDyn;
use lake::lake_view::LakeView;
use lake::{DropletBase, Lake};
use lake::lake::LakeAllocatorExt;

fn example_basic_split() {
    let mut lake: Lake<1024> = Lake::new();
    // Create a subview of 256 bytes from the main lake.
    let mut view: LakeView<1024> = lake.split(256).expect("not enough space in lake");
    // Allocate a fixed-size 64-byte droplet inside the view.
    let _droplet: Droplet<64, LakeView<1024>> =
        view.alloc::<64>().expect("not enough space in view");
    // The droplet is carved from the lake via the view, offset is local to the view.
    assert_eq!(view.used(), 64);
    assert_eq!(view.remaining(), 192); // 256 - 64
}

fn example_nested_split() {
    let mut lake: Lake<1024> = Lake::new();
    // Split off 512 bytes
    let mut parent_view: LakeView<1024> = lake.split(512).unwrap();
    // From the parent view, split another 128-byte view
    let mut child_view: LakeView<1024> = parent_view.split(128).unwrap();
    // Allocate something inside the child view
    let _droplet: Droplet<32, LakeView<1024>> = child_view.alloc::<32>().unwrap();
    assert_eq!(child_view.used(), 32);
    assert_eq!(child_view.remaining(), 96);
}

fn example_process_in_view() {
    let mut lake: Lake<1024> = Lake::new();
    // Split 300 bytes for this use case
    let mut view: LakeView<1024> = lake.split(300).unwrap();
    let dynamic_droplet: DropletDyn<1024> = view
        .process(|remaining| {
            let json = br#"{"ok":true}"#;
            assert!(json.len() <= remaining);
            json.to_vec()
        })
        .unwrap();
    let data: &[u8] = dynamic_droplet.d_as_slice();
    assert_eq!(data, b"{\"ok\":true}");
}

fn example_mark_reset_view() {
    let mut lake: Lake<1024> = Lake::new();
    let mut view: LakeView<1024> = lake.split(256).unwrap();
    view.mark(); // Save offset = 0
    let _ = view.alloc::<64>().unwrap();
    let _ = view.alloc::<32>().unwrap();
    assert_eq!(view.used(), 96);
    view.reset_to_mark(); // back to 0
    assert_eq!(view.used(), 0);
}

fn example_parallel_views() {
    let mut lake: Lake<1024> = Lake::new();
    let mut headers_view: LakeView<1024> = lake.split(128).unwrap();
    let mut body_view: LakeView<1024> = lake.split(512).unwrap();
    let mut metadata_view: LakeView<1024> = lake.split(64).unwrap();
    let _result1: DropletDyn<1024> = headers_view
        .process(|_| b"GET / HTTP/1.1\r\n\r\n".to_vec())
        .unwrap();
    let _result2: DropletDyn<1024> = body_view.process(|_| vec![0x42; 128]).unwrap();
    let _result3: DropletDyn<1024> = metadata_view.process(|_| vec![0x01, 0x02]).unwrap();
}

fn example_structs_in_view() {
    #[repr(C)]
    struct PacketHeader {
        kind: u8,
        flags: u8,
        length: u16,
    }
    let mut lake: Lake<1024> = Lake::new();
    let mut view: LakeView<1024> = lake.split(256).unwrap();
    // Allocate a single struct
    let header: &mut PacketHeader = view.alloc_struct();
    header.kind = 1;
    header.flags = 0;
    header.length = 128;
    // Allocate a slice of u32
    let values: &mut [u32] = view.alloc_slice(8);
    for i in 0..8 {
        values[i] = i as u32;
    }
    assert_eq!(values[3], 3);
}

fn example_zeroing_reset() {
    let mut lake: Lake<1024> = Lake::new();
    let mut view: LakeView<1024> = lake.split(128).unwrap();
    view.set_zeroing(true);
    let _ = view.alloc::<64>().unwrap();
    view.reset(); // This zeroes out the first 64 bytes
    assert_eq!(view.used(), 0);
}

fn main() {
    example_basic_split();
    example_nested_split();
    example_process_in_view();
    example_mark_reset_view();
    example_parallel_views();
    example_structs_in_view();
    example_zeroing_reset();
}
