use pretty_assertions::assert_eq;
use serde::{Deserialize, Serialize};

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

#[test]
fn enum_simple() {
    let expected = TestEnum::Empty;
    assert_same(&expected);
}

#[test]
fn enum_unint() {
    assert_same(&1_u32);
}

#[test]
fn enum_newtype() {
    let expected = TestEnum::NewType("HelloWorld".to_owned());
    assert_same(&expected);
}

#[test]
fn enum_newtype_u32() {
    let expected = TestEnum::NewTypeVariant(1);
    assert_same(&expected);
}

#[test]
// #[ignore = "TODO"]
fn enum_structtype() {
    let expected = TestEnum::StructType {
        inner: "HelloWorld".to_owned(),
    };
    assert_same(&expected);
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
    println!("our_bytes: {}", our_str);
    println!("their_bytes: {}", rmp_str);
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
