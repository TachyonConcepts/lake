use lake::droplet_dyn::DropletDyn;
use lake::lake::LakeError;
use lake::{DropletBase, Lake};

fn example_1() {
    let mut lake: Lake<1024> = Lake::new();

    // Compress some data using a mock encoder.
    let compressed: DropletDyn<1024> = lake
        .process(|_remaining| {
            let input = b"aaaaaaaaaaaaaaaaaaaaaabbbbbbbbbbbbbbbbcccccccc";
            // Fake compression: just truncate or repeat
            let mut out = Vec::new();
            for chunk in input.chunks(4) {
                out.push(chunk[0]); // pretend it's compressed
            }
            out
        })
        .expect("compression fits in lake");

    // Use the result
    let compressed_bytes = compressed.d_as_slice();
    println!("Compressed {} bytes", compressed_bytes.len());
}

fn example_2() {
    let mut lake: Lake<512> = Lake::new();
    let user_id = 42;
    let name = "Alice";
    // Simulate JSON encoding in-place
    let json: DropletDyn<512> = lake
        .process(|remaining| {
            let s = format!(r#"{{"id":{},"name":"{}"}}"#, user_id, name);
            assert!(s.len() <= remaining);
            s.into_bytes()
        })
        .unwrap();
    let output = std::str::from_utf8(json.d_as_slice()).unwrap();
    assert_eq!(output, r#"{"id":42,"name":"Alice"}"#);
}

fn example_3() {
    let mut lake: Lake<64> = Lake::new();
    let result: Result<DropletDyn<64>, LakeError> = lake.process(|remaining| {
        // Silly closure: ignore limit, return way too much
        vec![42u8; remaining + 1]
    });
    assert!(matches!(result, Err(LakeError::Overflow)));
}

fn example_4() {
    let mut lake: Lake<128> = Lake::new();
    // Simulate progressive fill: only fill as much as fits
    let droplet: DropletDyn<128> = lake
        .process(|space| {
            // generate exactly what fits
            let data = (0..space).map(|i| (i % 256) as u8).collect();
            data
        })
        .unwrap();
    assert_eq!(droplet.d_len(), 128);
}

fn example_5() {
    let mut lake: Lake<256> = Lake::new();
    let payload: DropletDyn<256> = lake
        .process(|_space| {
            let mut packet = Vec::with_capacity(32);
            packet.push(0xAB); // Start byte
            packet.push(0xCD); // Command ID
            packet.extend_from_slice(&[0x01, 0x02, 0x03, 0x04]); // payload
            packet.push(0xEF); // End byte
            packet
        })
        .unwrap();

    let bytes: &[u8] = payload.d_as_slice();
    assert_eq!(bytes[0], 0xAB);
    assert_eq!(bytes[1], 0xCD);
    assert_eq!(bytes.last(), Some(&0xEF));
}

fn main() {
    example_1();
    example_2();
    example_3();
    example_4();
    example_5();
}
