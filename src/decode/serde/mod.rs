use crate::marker::Marker;
use core::fmt;
use paste::paste;
use serde::de::{self, Visitor};

use self::{enum_::UnitVariantAccess, map::MapAccess, seq::SeqAccess};

mod enum_;
mod map;
mod seq;

use super::Error;

type Result<T> = core::result::Result<T, Error>;

#[cfg(debug_assertions)]
fn print_debug<T>(prefix: &str, function_name: &str, de: &Deserializer) {
    extern crate std;
    use std::println;
    println!(
        "{}{}<{}> ({:?})",
        prefix,
        function_name,
        core::any::type_name::<T>(),
        &de.slice[de.index..core::cmp::min(de.slice.len(), de.index + 10)]
    );
}
#[cfg(debug_assertions)]
fn print_debug_value<T, V: core::fmt::Debug>(function_name: &str, de: &Deserializer, value: &V) {
    extern crate std;
    use std::println;
    println!(
        "{}<{}> => {:?}   ({:?})",
        function_name,
        core::any::type_name::<T>(),
        value,
        &de.slice[de.index..core::cmp::min(de.slice.len(), de.index + 10)]
    );
}
#[cfg(not(debug_assertions))]
fn print_debug<T>(_prefix: &str, _function_name: &str, _de: &Deserializer) {}
#[cfg(not(debug_assertions))]
fn print_debug_value<T, V: core::fmt::Debug>(_function_name: &str, _de: &Deserializer, _value: &V) {}

pub(crate) struct Deserializer<'b> {
    slice: &'b [u8],
    index: usize,
}

impl<'a> Deserializer<'a> {
    pub fn new(slice: &'a [u8]) -> Deserializer<'_> { Deserializer { slice, index: 0 } }

    fn eat_byte(&mut self) { self.index += 1; }

    fn peek(&mut self) -> Option<Marker> { Some(Marker::from_u8(*self.slice.get(self.index)?)) }
}

// NOTE(deserialize_*signed) we avoid parsing into u64 and then casting to a smaller integer, which
// is what upstream does, to avoid pulling in 64-bit compiler intrinsics, which waste a few KBs of
// Flash, when targeting non 64-bit architectures
macro_rules! deserialize_primitive {
    ($ty:ident) => {
        paste! {
            fn [<deserialize_ $ty>]<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value>
        {
            print_debug::<V>("Deserializer::deserialize_", stringify!($ty), &self);
            let (value, len) = paste! { super::[<read_ $ty>](&self.slice[self.index..])? };
            self.index += len;
            print_debug_value::<$ty, $ty>(stringify!(concat_idents!(Deserializer::deserialize_, $ty)), &self, &value);
            paste! { visitor.[<visit_ $ty>](value) }
        }}
    };
}
macro_rules! deserialize_primitives {
    ($($ty:ident),*) => { $( deserialize_primitive!($ty); )* };
}

impl<'a, 'de> de::Deserializer<'de> for &'a mut Deserializer<'de> {
    type Error = Error;

    deserialize_primitives!(bool, u8, u16, u32, u64, i8, i16, i32, i64, f32, f64);

    fn deserialize_str<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value> {
        print_debug::<V>("Deserializer::deserialize_", "str", &self);
        let (s, len) = super::read_str(&self.slice[self.index..])?;
        self.index += len;
        visitor.visit_borrowed_str(s)
    }

    fn deserialize_bytes<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value> {
        print_debug::<V>("Deserializer::deserialize_", "bytes", &self);
        let (value, len) = super::read_bin(&self.slice[self.index..])?;
        self.index += len;
        visitor.visit_borrowed_bytes(value)
    }

    fn deserialize_byte_buf<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value> {
        print_debug::<V>("Deserializer::deserialize_", "byte_buf", &self);
        self.deserialize_bytes(visitor)
    }

    fn deserialize_option<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value> {
        print_debug::<V>("Deserializer::deserialize_", "option", &self);
        let marker = self.peek().ok_or(Error::EndOfBuffer)?;
        match marker {
            Marker::Null => {
                self.eat_byte();
                visitor.visit_none()
            }
            _ => visitor.visit_some(self),
        }
    }

    fn deserialize_seq<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value> {
        print_debug::<V>("Deserializer::deserialize_", "seq", &self);
        let (len, header_len) = crate::decode::read_array_len(&self.slice[self.index..])?;
        self.index += header_len;
        visitor.visit_seq(SeqAccess::new(self, len))
    }

    fn deserialize_tuple<V: Visitor<'de>>(self, _len: usize, visitor: V) -> Result<V::Value> {
        print_debug::<V>("Deserializer::deserialize_", "tuple", &self);
        self.deserialize_seq(visitor)
    }

    fn deserialize_tuple_struct<V: Visitor<'de>>(self, _name: &'static str, _len: usize, visitor: V) -> Result<V::Value> {
        print_debug::<V>("Deserializer::deserialize_", "tuple_struct", &self);
        self.deserialize_seq(visitor)
    }

    fn deserialize_map<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value> {
        print_debug::<V>("Deserializer::deserialize_", "map", &self);
        let (len, header_len) = crate::decode::read_map_len(&self.slice[self.index..])?;
        self.index += header_len;
        visitor.visit_map(MapAccess::new(self, len))
    }

    fn deserialize_struct<V: Visitor<'de>>(self, _name: &'static str, _fields: &'static [&'static str], visitor: V) -> Result<V::Value> {
        print_debug::<V>("Deserializer::deserialize_", "struct", &self);
        self.deserialize_map(visitor)
    }

    fn deserialize_enum<V: Visitor<'de>>(self, _name: &'static str, _variants: &'static [&'static str], visitor: V) -> Result<V::Value> {
        print_debug::<V>("Deserializer::deserialize_", "enum", &self);
        visitor.visit_enum(UnitVariantAccess::new(self))
    }

    fn deserialize_identifier<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value> {
        print_debug::<V>("Deserializer::deserialize_", "identifier", &self);
        self.deserialize_str(visitor)
    }

    /// Unsupported. Can’t parse a value without knowing its expected type.
    fn deserialize_any<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value> {
        print_debug::<V>("Deserializer::deserialize_", "any", &self);
        match self.peek() {
            Some(Marker::FixPos(_)) | Some(Marker::U8) => self.deserialize_u8(visitor),
            Some(Marker::U16) => self.deserialize_u16(visitor),
            Some(Marker::U32) => self.deserialize_u32(visitor),
            Some(Marker::U64) => self.deserialize_u64(visitor),

            Some(Marker::FixNeg(_)) | Some(Marker::I8) => self.deserialize_i8(visitor),
            Some(Marker::I16) => self.deserialize_i16(visitor),
            Some(Marker::I32) => self.deserialize_i32(visitor),
            Some(Marker::I64) => self.deserialize_i64(visitor),

            Some(Marker::F32) => self.deserialize_f32(visitor),
            Some(Marker::F64) => self.deserialize_f64(visitor),

            Some(Marker::Null) => self.deserialize_option(visitor),

            Some(Marker::True) => self.deserialize_bool(visitor),
            Some(Marker::False) => self.deserialize_bool(visitor),

            Some(Marker::FixStr(_)) | Some(Marker::Str8) | Some(Marker::Str16) | Some(Marker::Str32) => self.deserialize_str(visitor),
            Some(Marker::Bin8) | Some(Marker::Bin16) | Some(Marker::Bin32) => self.deserialize_bytes(visitor),

            Some(Marker::FixArray(_)) | Some(Marker::Array16) | Some(Marker::Array32) => self.deserialize_seq(visitor),
            Some(Marker::FixMap(_)) | Some(Marker::Map16) | Some(Marker::Map32) => self.deserialize_map(visitor),

            Some(Marker::FixExt1)
            | Some(Marker::FixExt2)
            | Some(Marker::FixExt4)
            | Some(Marker::FixExt8)
            | Some(Marker::FixExt16)
            | Some(Marker::Ext8)
            | Some(Marker::Ext16)
            | Some(Marker::Ext32) => Err(Error::InvalidType),

            Some(Marker::Reserved) => Err(Error::InvalidType),
            None => visitor.visit_unit(),
        }
    }

    /// Used to throw out fields that we don’t want to keep in our structs.
    fn deserialize_ignored_any<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value> {
        print_debug::<V>("Deserializer::deserialize_", "ignored_any", &self);
        self.deserialize_any(visitor)
    }

    /// Unsupported. Use a more specific deserialize_* method
    fn deserialize_unit<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value> {
        print_debug::<V>("Deserializer::deserialize_", "unit", &self);
        let marker = self.peek().ok_or(Error::EndOfBuffer)?;
        match marker {
            Marker::Null | Marker::FixArray(0) => {
                self.eat_byte();
                visitor.visit_unit()
            }
            _ => Err(Error::InvalidType),
        }
    }

    /// Unsupported. Use a more specific deserialize_* method
    fn deserialize_unit_struct<V: Visitor<'de>>(self, _name: &'static str, visitor: V) -> Result<V::Value> {
        print_debug::<V>("Deserializer::deserialize_", "unit_struct", &self);
        self.deserialize_unit(visitor)
    }

    fn deserialize_char<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value> {
        print_debug::<V>("Deserializer::deserialize_", "char", &self);
        //TODO Need to decide how to encode this. Probably as a str?
        self.deserialize_str(visitor)
    }

    /// Unsupported. We can’t parse newtypes because we don’t know the underlying type.
    fn deserialize_newtype_struct<V: Visitor<'de>>(self, _name: &'static str, _visitor: V) -> Result<V::Value> {
        print_debug::<V>("Deserializer::deserialize_", "newtype_struct", &self);
        unreachable!()
    }

    /// Unsupported. String is not available in no-std.
    fn deserialize_string<V: Visitor<'de>>(self, _visitor: V) -> Result<V::Value> {
        print_debug::<V>("Deserializer::deserialize_", "string", &self);
        unreachable!()
    }
}

impl de::Error for Error {
    #[cfg_attr(not(feature = "custom-error-messages"), allow(unused_variables))]
    fn custom<T>(msg: T) -> Self
    where T: fmt::Display {
        #[cfg(not(feature = "custom-error-messages"))]
        {
            Error::CustomError
        }
        #[cfg(feature = "custom-error-messages")]
        {
            use core::fmt::Write;

            let mut string = heapless::String::new();
            write!(string, "{:.64}", msg).unwrap();
            Error::CustomErrorWithMessage(string)
        }
    }
}

impl fmt::Display for Error {
    #[cfg(debug_assertions)]
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{}",
            match self {
                Error::InvalidType => "Unexpected type encountered.",
                Error::OutOfBounds => "Index out of bounds.",
                Error::EndOfBuffer => "EOF while parsing.",
                Error::CustomError => "Did not match deserializer’s expected format.",
                #[cfg(feature = "custom-error-messages")]
                Error::CustomErrorWithMessage(msg) => msg.as_str(),
                // _ => "Invalid MessagePack",
            }
        )
    }
    #[cfg(not(debug_assertions))]
    fn fmt(&self, _f: &mut fmt::Formatter<'_>) -> fmt::Result { Ok(()) }
}