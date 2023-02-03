// Found when trying to decode data from this benchmark : https://github.com/cloudflare/serde-wasm-bindgen/tree/master/benchmarks
#[test]
fn serde_json_value() {
    let bytes = b"\x81\xa6source\xa9zip64.zip";
    let actual: serde_json::Value = wasm_msgpack::decode::from_slice(bytes).unwrap();
    let expected = serde_json::json!({
      "source": "zip64.zip"
    });
    assert_eq!(expected, actual);
}
