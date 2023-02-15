#[cfg(feature = "serde")]
mod decode;
mod encode;
#[cfg(feature = "serde")]
mod fuzzing;
#[cfg(all(feature = "serde"))]
mod interop;
#[cfg(all(feature = "serde", feature = "compliant"))]
mod regression;
#[cfg(all(feature = "serde"))]
mod roundtrip;
