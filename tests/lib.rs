#[cfg(feature = "serde")]
mod decode;
mod encode;
#[cfg(feature = "serde")]
mod fuzzing;
#[cfg(all(feature = "serde", feature = "std"))]
mod interop;
#[cfg(all(feature = "serde", feature = "compliant", feature = "std"))]
mod regression;
#[cfg(all(feature = "serde", feature = "std"))]
mod roundtrip;
