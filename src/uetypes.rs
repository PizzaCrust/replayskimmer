use serde::export::fmt::Debug;
use serde::export::Formatter;
use serde::{Deserialize, Deserializer};
use serde::de::{Visitor, SeqAccess, Error};

#[derive(Debug, Default, PartialEq)]
pub struct GUID(String);

struct GUIDVisitor;
impl<'de> Visitor<'de> for GUIDVisitor {
    type Value = GUID;

    fn expecting(&self, formatter: &mut Formatter) -> std::fmt::Result {
        formatter.write_str("16 byte guid")
    }

    fn visit_seq<A>(self, mut seq: A) -> Result<Self::Value, <A as SeqAccess<'de>>::Error> where
        A: SeqAccess<'de>, {
        let mut bytes = [0u8; 16];
        for i in 0..16 {
            bytes[i] = seq.next_element::<u8>()?.ok_or_else(|| A::Error::custom(""))?;
        }
        Ok(GUID(hex::encode(bytes)))
    }
}

impl<'de> Deserialize<'de> for GUID {
    fn deserialize<D>(deserializer: D) -> Result<Self, <D as Deserializer<'de>>::Error> where
        D: Deserializer<'de> {
        deserializer.deserialize_tuple(16, GUIDVisitor)
    }
}