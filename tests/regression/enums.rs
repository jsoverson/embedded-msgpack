use serde::{de::DeserializeOwned, Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[must_use]
pub enum TestEnum {
    Empty,
    NewType(String),
    StructType { inner: String },
}

#[test]
fn enum_simple() {
    let expected = TestEnum::Empty;
    assert_rt(&expected);
}
#[test]
fn enum_newtype() {
    let expected = TestEnum::NewType("HelloWorld".to_owned());
    assert_rt(&expected);
}
#[test]
fn enum_structtype() {
    let expected = TestEnum::StructType {
        inner: "HelloWorld".to_owned(),
    };
    assert_rt(&expected);
}

fn assert_rt<T>(expected: &T)
where
    T: ?Sized + Serialize + DeserializeOwned + std::fmt::Debug + PartialEq + 'static,
{
    let our_bytes = serialize(expected);
    let our_str = our_bytes
        .iter()
        .map(|a| String::from_utf8(vec![*a]).unwrap_or_else(|_| format!("\\{:02x?}", *a)))
        .collect::<Vec<String>>()
        .join("");
    println!("our_bytes: {}", our_str);
    let actual: T = wasm_msgpack::decode::from_slice(&our_bytes).unwrap();
    assert_eq!(expected, &actual);
}

fn serialize<T>(item: &T) -> Vec<u8>
where
    T: ?Sized + Serialize,
{
    let mut buf = [0; 1024 * 100];
    let written = wasm_msgpack::encode::serde::to_array(item, &mut buf).unwrap();
    buf[0..written].to_vec()
}
