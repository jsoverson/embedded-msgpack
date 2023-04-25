// Found when trying to decode data from this benchmark : https://github.com/cloudflare/serde-wasm-bindgen/tree/master/benchmarks

use std::collections::BTreeMap;

#[test]
fn regression_json_null() {
    let bytes = r#"{"nullval":null}"#;
    let json: serde_json::Value = serde_json::from_str(bytes).unwrap();
    let encoded = serialize(&json);
    let decoded: serde_json::Value = wasm_msgpack::decode::from_slice(&encoded).unwrap();
    assert_eq!(json, decoded);
}

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

fn serialize<T>(item: &T) -> Vec<u8>
where
    T: ?Sized + serde::Serialize,
{
    let mut buf = [0; 1024 * 100];
    let written = wasm_msgpack::encode::serde::to_array(item, &mut buf).unwrap();
    buf[0..written].to_vec()
}
