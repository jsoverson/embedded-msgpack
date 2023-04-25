use pretty_assertions::assert_eq;
use serde::{Deserialize, Serialize};
use wasm_msgpack::decode;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[must_use]
pub enum TestEnum {
    Empty,
    NewType(String),
    StructType { inner: String },
    UnitVariant,
    NewTypeVariant(i32),
    TupleVariant(i32, u8),
    StructVariant { a: i32, b: u8 },
}

#[rstest::rstest]
#[case(TestEnum::Empty)]
#[case(&1_u32)]
#[case(&123_u8)]
#[case(&TestEnum::NewType("HelloWorld".to_owned()))]
#[case(&TestEnum::NewTypeVariant(1))]
#[case(&TestEnum::StructType {
    inner: "HelloWorld".to_owned()
})]

fn test<T>(#[case] item: T)
where
    T: Serialize,
    T: std::fmt::Debug,
{
    assert_same(&item);
}

#[test]
fn json_null() {
    let bytes = r#"{"nullval":null}"#;
    let json: serde_json::Value = serde_json::from_str(bytes).unwrap();
    assert_same(&json);
}
#[test]
fn small_num_json() {
    let bytes = serialize(&10);
    let json: serde_json::Value = decode::from_slice(&bytes).unwrap();
    assert_eq!(json, serde_json::json!(10));
}

fn assert_same<T>(item: &T)
where
    T: ?Sized + Serialize,
{
    let our_bytes = serialize(item);
    let rmp_bytes = rmp_serialize(item);
    let our_str = our_bytes
        .iter()
        .map(|a| String::from_utf8(vec![*a]).unwrap_or_else(|_| format!("\\{:02x?}", *a)))
        .collect::<Vec<String>>()
        .join("");
    let rmp_str = rmp_bytes
        .iter()
        .map(|a| String::from_utf8(vec![*a]).unwrap_or_else(|_| format!("\\{:02x?}", *a)))
        .collect::<Vec<String>>()
        .join("");
    println!("our byte str: {}", our_str);
    println!("their byte str: {}", rmp_str);
    println!("our_bytes: {:?}", our_bytes);
    println!("their_bytes: {:?}", rmp_bytes);
    assert_eq!(our_str, rmp_str);
}

fn rmp_serialize<T>(item: &T) -> Vec<u8>
where
    T: ?Sized + Serialize,
{
    let mut buff = Vec::new();
    let mut serializer = rmp_serde::encode::Serializer::new(&mut buff).with_binary().with_struct_tuple();
    item.serialize(&mut serializer).unwrap();
    buff
}

fn serialize<T>(item: &T) -> Vec<u8>
where
    T: ?Sized + Serialize,
{
    let mut buf = [0; 1024 * 100];
    let written = wasm_msgpack::encode::serde::to_array(item, &mut buf).unwrap();
    buf[0..written].to_vec()
}
