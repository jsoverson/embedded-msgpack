use serde::{Deserialize, Serialize};
use std::collections::BTreeMap as Map;

pub type Canada = FeatureCollection;

#[derive(Serialize, Deserialize, Debug, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct FeatureCollection {
    #[serde(rename = "type")]
    pub obj_type: ObjType,
    pub features: Vec<Feature>,
}

#[derive(Serialize, Deserialize, Debug, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct Feature {
    #[serde(rename = "type")]
    pub obj_type: ObjType,
    pub properties: Map<String, String>,
    pub geometry: Geometry,
}

#[derive(Serialize, Deserialize, Debug, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct Geometry {
    #[serde(rename = "type")]
    pub obj_type: ObjType,
    pub coordinates: Vec<Vec<(Latitude, Longitude)>>,
}

pub type Latitude = f32;
pub type Longitude = f32;

#[derive(Serialize, Deserialize, Debug, PartialEq)]
#[serde(deny_unknown_fields)]
pub enum ObjType {
    FeatureCollection,
    Feature,
    Polygon,
}

const OBJ_A: [u8; 375] = [
    130, 164, 116, 121, 112, 101, 177, 70, 101, 97, 116, 117, 114, 101, 67, 111, 108, 108, 101, 99, 116, 105, 111, 110, 168, 102, 101, 97,
    116, 117, 114, 101, 115, 145, 131, 164, 116, 121, 112, 101, 167, 70, 101, 97, 116, 117, 114, 101, 170, 112, 114, 111, 112, 101, 114,
    116, 105, 101, 115, 129, 164, 110, 97, 109, 101, 166, 67, 97, 110, 97, 100, 97, 168, 103, 101, 111, 109, 101, 116, 114, 121, 130, 164,
    116, 121, 112, 101, 167, 80, 111, 108, 121, 103, 111, 110, 171, 99, 111, 111, 114, 100, 105, 110, 97, 116, 101, 115, 145, 158, 146,
    203, 192, 80, 103, 69, 128, 60, 209, 64, 203, 64, 69, 181, 203, 129, 115, 50, 40, 146, 203, 192, 80, 103, 169, 126, 19, 43, 88, 203,
    64, 69, 181, 130, 194, 189, 127, 80, 146, 203, 192, 80, 104, 0, 0, 0, 0, 0, 203, 64, 69, 181, 239, 191, 64, 28, 88, 146, 203, 192, 80,
    104, 182, 65, 112, 12, 208, 203, 64, 69, 185, 144, 66, 216, 194, 160, 146, 203, 192, 80, 104, 132, 1, 129, 224, 60, 203, 64, 69, 188,
    195, 67, 183, 15, 8, 146, 203, 192, 80, 103, 32, 255, 84, 8, 152, 203, 64, 69, 193, 171, 192, 227, 138, 136, 146, 203, 192, 80, 102,
    198, 0, 41, 241, 108, 203, 64, 69, 194, 15, 194, 235, 162, 120, 146, 203, 192, 80, 102, 75, 64, 112, 50, 152, 203, 64, 69, 194, 6, 192,
    13, 161, 160, 146, 203, 192, 80, 100, 58, 255, 176, 78, 232, 203, 64, 69, 193, 16, 253, 126, 69, 136, 146, 203, 192, 80, 99, 246, 190,
    55, 222, 148, 203, 64, 69, 192, 145, 125, 107, 101, 168, 146, 203, 192, 80, 99, 215, 1, 217, 244, 208, 203, 64, 69, 191, 246, 194, 105,
    156, 136, 146, 203, 192, 80, 100, 177, 129, 22, 235, 212, 203, 64, 69, 188, 249, 252, 176, 192, 48, 146, 203, 192, 80, 101, 253, 193,
    97, 94, 188, 203, 64, 69, 184, 218, 1, 104, 181, 208, 146, 203, 192, 80, 103, 69, 128, 60, 209, 64, 203, 64, 69, 181, 203, 129, 115,
    50, 40,
];

// Found when trying to decode data from this benchmark : https://github.com/cloudflare/serde-wasm-bindgen/tree/master/benchmarks
#[test]
#[allow(clippy::excessive_precision)]
fn serde_wasm_bench() {
    let actual: Canada = wasm_msgpack::decode::from_slice(&OBJ_A).unwrap();
    let expected = Canada {
        obj_type: ObjType::FeatureCollection,
        features: vec![Feature {
            obj_type: ObjType::Feature,
            properties: Map::from([("name".to_owned(), "Canada".to_owned())]),
            geometry: Geometry {
                obj_type: ObjType::Polygon,
                coordinates: vec![vec![
                    (-65.613616999999977, 43.420273000000009),
                    (-65.619720000000029, 43.418052999999986),
                    (-65.625, 43.421379000000059),
                    (-65.636123999999882, 43.449714999999969),
                    (-65.633056999999951, 43.474709000000132),
                    (-65.611389000000031, 43.513054000000068),
                    (-65.605835000000013, 43.516105999999979),
                    (-65.598343, 43.515830999999935),
                    (-65.566101000000003, 43.508331000000055),
                    (-65.561935000000005, 43.504439999999988),
                    (-65.55999799999995, 43.499718000000087),
                    (-65.573333999999988, 43.476379000000065),
                    (-65.593612999999948, 43.444153000000028),
                    (-65.613616999999977, 43.42027300000000),
                ]],
            },
        }],
    };
    assert_eq!(actual, expected);
}
