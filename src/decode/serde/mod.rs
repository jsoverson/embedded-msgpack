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

#[cfg(test)]
fn print_debug<T>(prefix: &str, function_name: &str, de: &Deserializer) {
    #[cfg(not(feature = "std"))]
    extern crate std;
    #[cfg(not(feature = "std"))]
    use std::println;
    println!(
        "{}{}<{}> ({:02x?})",
        prefix,
        function_name,
        core::any::type_name::<T>(),
        &de.slice[de.index..core::cmp::min(de.slice.len(), de.index + 10)]
    );
}

#[cfg(test)]
fn print_debug_value<T, V: core::fmt::Debug>(function_name: &str, de: &Deserializer, value: &V) {
    #[cfg(not(feature = "std"))]
    extern crate std;
    #[cfg(not(feature = "std"))]
    use std::println;
    println!(
        "{}<{}> => {:?}   ({:02x?})",
        function_name,
        core::any::type_name::<T>(),
        value,
        &de.slice[de.index..core::cmp::min(de.slice.len(), de.index + 10)]
    );
}

#[cfg(not(test))]
#[allow(clippy::missing_const_for_fn)]
fn print_debug<T>(_prefix: &str, _function_name: &str, _de: &Deserializer) {}
#[cfg(not(test))]
#[allow(clippy::missing_const_for_fn)]
fn print_debug_value<T, V: core::fmt::Debug>(_function_name: &str, _de: &Deserializer, _value: &V) {}

pub(crate) struct Deserializer<'b> {
    slice: &'b [u8],
    index: usize,
    state: State,
}

impl<'a> Deserializer<'a> {
    pub const fn new(slice: &'a [u8]) -> Deserializer<'_> {
        Deserializer {
            slice,
            index: 0,
            state: State::Normal,
        }
    }

    fn eat_byte(&mut self) {
        self.index += 1;
    }

    fn peek(&mut self) -> Option<Marker> {
        Some(Marker::from_u8(*self.slice.get(self.index)?))
    }
}

macro_rules! deserialize_primitives {
    ($into:ident, $($ty:ident),*) => {
      $(paste! {
        fn [<deserialize_ $ty>]<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value>
        { paste! { self.[<deserialize_ $into>](visitor) } }
       })*
    };
}

enum State {
    Normal,
    Ext(usize),
}

impl<'a, 'de> de::Deserializer<'de> for &'a mut Deserializer<'de> {
    type Error = Error;

    deserialize_primitives!(i64, i16, i32);
    deserialize_primitives!(u64, u8, u16, u32);
    deserialize_primitives!(f64, f32);

    fn deserialize_i8<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        print_debug::<V>("Deserializer::deserialize_", "i8", self);
        let (value, len) = match self.state {
            State::Normal => super::read_i8(&self.slice[self.index..])?,
            // read the ext type as raw byte and not encoded as a normal i8
            #[cfg(feature = "ext")]
            State::Ext(_) => (self.slice[self.index] as i8, 1),
        };
        self.index += len;
        print_debug_value::<i8, i8>("Deserializer::deserialize_i8", self, &value);
        visitor.visit_i8(value)
    }

    fn deserialize_str<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value> {
        print_debug::<V>("Deserializer::deserialize_", "str", self);
        let (s, len) = super::read_str(&self.slice[self.index..])?;
        self.index += len;
        visitor.visit_borrowed_str(s)
    }

    fn deserialize_bytes<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value> {
        print_debug::<V>("Deserializer::deserialize_", "bytes", self);
        let (value, len) = match self.state {
            State::Normal => super::read_bin(&self.slice[self.index..])?,
            // read the ext type as raw byte and not encoded as a normal i8
            #[cfg(feature = "ext")]
            State::Ext(len) => {
                self.state = State::Normal;
                (&self.slice[self.index..self.index + len], len)
            }
        };
        self.index += len;
        visitor.visit_borrowed_bytes(value)
    }

    fn deserialize_byte_buf<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value> {
        print_debug::<V>("Deserializer::deserialize_", "byte_buf", self);
        self.deserialize_bytes(visitor)
    }

    fn deserialize_option<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value> {
        print_debug::<V>("Deserializer::deserialize_", "option", self);
        let marker = self.peek().ok_or(Error::EndOfBuffer(Marker::Reserved))?;
        match marker {
            Marker::Null => {
                self.eat_byte();
                visitor.visit_none()
            }
            _ => visitor.visit_some(self),
        }
    }

    fn deserialize_seq<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value> {
        print_debug::<V>("Deserializer::deserialize_", "seq", self);
        let (len, header_len) = crate::decode::read_array_len(&self.slice[self.index..])?;
        self.index += header_len;
        visitor.visit_seq(SeqAccess::new(self, len))
    }

    fn deserialize_tuple<V: Visitor<'de>>(self, _len: usize, visitor: V) -> Result<V::Value> {
        print_debug::<V>("Deserializer::deserialize_", "tuple", self);
        self.deserialize_seq(visitor)
    }

    fn deserialize_tuple_struct<V: Visitor<'de>>(self, _name: &'static str, _len: usize, visitor: V) -> Result<V::Value> {
        print_debug::<V>("Deserializer::deserialize_", "tuple_struct", self);
        self.deserialize_seq(visitor)
    }

    fn deserialize_map<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value> {
        print_debug::<V>("Deserializer::deserialize_", "map", self);
        let (len, header_len) = crate::decode::read_map_len(&self.slice[self.index..])?;
        self.index += header_len;
        visitor.visit_map(MapAccess::new(self, len))
    }

    fn deserialize_struct<V: Visitor<'de>>(self, name: &'static str, _fields: &'static [&'static str], visitor: V) -> Result<V::Value> {
        print_debug::<V>("Deserializer::deserialize_", "struct", self);
        match name {
            #[cfg(feature = "ext")]
            crate::ext::TYPE_NAME | crate::timestamp::TYPE_NAME => {
                if let Some(marker) = self.peek() {
                    match marker {
                        Marker::FixExt1
                        | Marker::FixExt2
                        | Marker::FixExt4
                        | Marker::FixExt8
                        | Marker::FixExt16
                        | Marker::Ext8
                        | Marker::Ext16
                        | Marker::Ext32 => {
                            let (header_len, data_len) = crate::ext::read_ext_len(&self.slice[self.index..])?;
                            self.index += header_len - 1; // move forward minus 1 byte for the ext type (header_len includes the type byte)
                            self.state = State::Ext(data_len);
                            visitor.visit_seq(SeqAccess::new(self, 2))
                        }
                        _ => Err(Error::InvalidType),
                    }
                } else {
                    Err(Error::EndOfBuffer(Marker::Reserved))
                }
            }
            _ => self.deserialize_map(visitor),
        }
    }

    fn deserialize_enum<V: Visitor<'de>>(self, _name: &'static str, _variants: &'static [&'static str], visitor: V) -> Result<V::Value> {
        print_debug::<V>("Deserializer::deserialize_", "enum", self);
        visitor.visit_enum(UnitVariantAccess::new(self))
    }

    fn deserialize_identifier<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value> {
        print_debug::<V>("Deserializer::deserialize_", "identifier", self);
        let marker = self.peek().ok_or(Error::EndOfBuffer(Marker::Reserved))?;
        #[allow(clippy::single_match)]
        match marker {
            Marker::FixMap(_) => {
                let (_len, header_len) = crate::decode::read_map_len(&self.slice[self.index..])?;
                self.index += header_len;
            }
            _ => {}
        }
        self.deserialize_str(visitor)
    }

    /// Unsupported. Can’t parse a value without knowing its expected type.
    fn deserialize_any<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value> {
        let marker = self.peek().ok_or(Error::EndOfBuffer(Marker::Reserved))?;
        match marker {
            Marker::FixPos(_) => self.deserialize_u8(visitor),
            Marker::FixMap(_) => self.deserialize_map(visitor),
            Marker::Map16 => self.deserialize_map(visitor),
            Marker::Map32 => self.deserialize_map(visitor),
            Marker::FixArray(_) => self.deserialize_seq(visitor),
            Marker::Array16 => self.deserialize_seq(visitor),
            Marker::Array32 => self.deserialize_seq(visitor),
            Marker::Str8 => self.deserialize_str(visitor),
            Marker::Str16 => self.deserialize_str(visitor),
            Marker::Str32 => self.deserialize_str(visitor),
            Marker::Bin8 => self.deserialize_bytes(visitor),
            Marker::Bin16 => self.deserialize_bytes(visitor),
            Marker::Bin32 => self.deserialize_bytes(visitor),
            Marker::FixStr(_) => self.deserialize_str(visitor),
            Marker::F32 => self.deserialize_f32(visitor),
            Marker::F64 => self.deserialize_f64(visitor),
            Marker::I16 => self.deserialize_i16(visitor),
            Marker::I32 => self.deserialize_i32(visitor),
            Marker::I64 => self.deserialize_i64(visitor),
            Marker::I8 => self.deserialize_i8(visitor),
            Marker::U16 => self.deserialize_u16(visitor),
            Marker::U32 => self.deserialize_u32(visitor),
            Marker::U64 => self.deserialize_u64(visitor),
            Marker::U8 => self.deserialize_u8(visitor),
            Marker::True => self.deserialize_bool(visitor),
            Marker::False => self.deserialize_bool(visitor),
            _ => {
                print_debug::<V>("Deserializer::deserialize_", "any", self);
                let (_, n) = super::skip_any(&self.slice[self.index..])?;
                self.index += n;
                visitor.visit_unit()
            }
        }
    }

    /// Used to throw out fields that we don’t want to keep in our structs.
    fn deserialize_ignored_any<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value> {
        print_debug::<V>("Deserializer::deserialize_", "ignored_any", self);
        self.deserialize_any(visitor)
    }

    /// Unsupported. Use a more specific deserialize_* method
    fn deserialize_unit<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value> {
        print_debug::<V>("Deserializer::deserialize_", "unit", self);
        let marker = self.peek().ok_or(Error::EndOfBuffer(Marker::Reserved))?;
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
        print_debug::<V>("Deserializer::deserialize_", "unit_struct", self);
        self.deserialize_unit(visitor)
    }

    fn deserialize_char<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value> {
        print_debug::<V>("Deserializer::deserialize_", "char", self);
        //TODO Need to decide how to encode this. Probably as a str?
        self.deserialize_str(visitor)
    }

    fn deserialize_newtype_struct<V: Visitor<'de>>(self, _name: &'static str, visitor: V) -> Result<V::Value> {
        print_debug::<V>("Deserializer::deserialize_", "newtype_struct", self);
        visitor.visit_newtype_struct(self)
    }

    fn deserialize_string<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value> {
        print_debug::<V>("Deserializer::deserialize_", "string", self);
        self.deserialize_str(visitor)
    }

    fn deserialize_i64<V>(self, visitor: V) -> core::result::Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        print_debug::<V>("Deserializer::deserialize_", "i64", self);
        let (value, len) = super::read_i64(&self.slice[self.index..])?;
        self.index += len;
        print_debug_value::<i64, i64>("Deserializer::deserialize_i64", self, &value);
        visitor.visit_i64(value)
    }

    fn deserialize_u64<V>(self, visitor: V) -> core::result::Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        print_debug::<V>("Deserializer::deserialize_", "u64", self);
        let (value, len) = super::read_u64(&self.slice[self.index..])?;
        self.index += len;
        print_debug_value::<u64, u64>("Deserializer::deserialize_u64", self, &value);
        visitor.visit_u64(value)
    }

    fn deserialize_f64<V>(self, visitor: V) -> core::result::Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        print_debug::<V>("Deserializer::deserialize_", "f64", self);
        let (value, len) = super::read_f64(&self.slice[self.index..])?;
        self.index += len;
        print_debug_value::<f64, f64>("Deserializer::deserialize_f64", self, &value);
        visitor.visit_f64(value)
    }

    fn deserialize_bool<V>(self, visitor: V) -> core::result::Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        print_debug::<V>("Deserializer::deserialize_", "bool", self);
        let (value, len) = super::read_bool(&self.slice[self.index..])?;
        self.index += len;
        print_debug_value::<bool, bool>("Deserializer::deserialize_bool", self, &value);
        visitor.visit_bool(value)
    }
}

impl ::serde::de::StdError for Error {}
impl de::Error for Error {
    #[cfg_attr(not(feature = "custom-error-messages"), allow(unused_variables))]
    fn custom<T>(msg: T) -> Self
    where
        T: fmt::Display,
    {
        #[cfg(not(feature = "custom-error-messages"))]
        {
            Error::CustomError
        }
        #[cfg(all(not(feature = "std"), feature = "custom-error-messages"))]
        {
            use core::fmt::Write;

            let mut string = heapless::String::new();
            write!(string, "{:.512}", msg).unwrap();
            Error::CustomErrorWithMessage(string)
        }
        #[cfg(all(feature = "std", feature = "custom-error-messages"))]
        {
            Error::CustomErrorWithMessage(msg.to_string())
        }
    }
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        #[cfg(feature = "std")]
        let s;
        write!(
            f,
            "{}",
            match self {
                Error::InvalidType => "Unexpected type encountered.",
                Error::OutOfBounds => "Index out of bounds.",
                #[cfg(not(feature = "std"))]
                Error::EndOfBuffer(_) => "End of buffer reached.",
                #[cfg(feature = "std")]
                Error::EndOfBuffer(m) => {
                    s = format!("End of buffer reached: {}", u8::from(*m));
                    s.as_str()
                }
                Error::CustomError => "Did not match deserializer's expected format.",
                #[cfg(feature = "custom-error-messages")]
                Error::CustomErrorWithMessage(msg) => msg.as_str(),
                Error::NotAscii => "String contains non-ascii chars.",
                Error::InvalidBoolean => "Invalid boolean marker.",
                Error::InvalidBinType => "Invalid binary marker.",
                Error::InvalidStringType => "Invalid string marker.",
                Error::InvalidArrayType => "Invalid array marker.",
                Error::InvalidMapType => "Invalid map marker.",
                Error::InvalidNewTypeLength => "Invalid array length for newtype.",
                Error::InvalidUtf8(_) => "Invalid Utf8.",
            }
        )
    }
}
