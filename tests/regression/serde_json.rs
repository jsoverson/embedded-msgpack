// Found when trying to decode data from this benchmark : https://github.com/cloudflare/serde-wasm-bindgen/tree/master/benchmarks

use std::collections::BTreeMap;

#[test]
fn serde_json_value() {
    let bytes = b"\x81\xa6source\xa9zip64.zip";
    let actual: serde_json::Value = wasm_msgpack::decode::from_slice(bytes).unwrap();
    let expected = serde_json::json!({
      "source": "zip64.zip"
    });
    assert_eq!(expected, actual);
}

#[test]
fn btree_serde_value() {
    let bytes = b"\x82\xa1a\xa12\xa5input\xd9!https://google.com/path/here.html";
    let actual: BTreeMap<String, serde_json::Value> = wasm_msgpack::decode::from_slice(bytes).unwrap();
    let mut expected = BTreeMap::default();
    expected.insert("a".to_string(), serde_json::Value::String("2".to_string()));
    expected.insert(
        "input".to_string(),
        serde_json::Value::String("https://google.com/path/here.html".to_string()),
    );

    assert_eq!(expected, actual);
}
