use serde::{Deserialize, Deserializer};
use serde::de::{Visitor, Error, SeqAccess, EnumAccess, MapAccess};
use serde::export::Formatter;
use byteorder::{ReadBytesExt, LE};
use std::io::{Cursor, Read};
use std::convert::TryInto;

#[derive(Debug, PartialEq)]
pub struct FStr(String);

fn access_byte<'de, A: SeqAccess<'de>>(access: &mut A) -> Result<u8, A::Error> {
    access.next_element()?.ok_or_else(|| serde::de::Error::custom("couldn't grab next byte"))
}

fn access_many<'de, A: SeqAccess<'de>>(access: &mut A, len: usize) -> Result<Vec<u8>, A::Error> {
    let mut vec = vec![0u8; len];
    for i in 0..len {
        vec[i] = access_byte(access)?;
    }
    return Ok(vec)
}

// REALLY inefficient and horrible code, some errors just panic directly!
fn access_fstr<'de, A: SeqAccess<'de>>(access: &mut A) -> Result<FStr, A::Error> {
    let len_bytes = access_many(access,  4)?;
    let mut len = Cursor::new(len_bytes.as_slice()).read_i32::<LE>().expect("");
    let is_unicode = len < 0;
    if is_unicode {
        len *= -1;
    }
    if len < 0 {
        return Err(A::Error::custom("Archive corrupted"));
    }
    return match is_unicode {
        true => {
            let mut c = Cursor::new(access_many(access, (len * 2) as usize).expect(""));
            let mut u16_bytes = vec![0u16; len as usize];
            c.read_u16_into::<LE>(u16_bytes.as_mut_slice());
            Ok(FStr(String::from_utf16(u16_bytes.as_slice()).expect("").trim_matches(char::from(0))
                .trim_matches('\u{0020}').to_string()))
        }
        false => {
            let mut c = Cursor::new(access_many(access, len as usize)?);
            let mut u8_bytes = vec![0u8; len as usize];
            c.read(u8_bytes.as_mut_slice());
            Ok(FStr(std::str::from_utf8(u8_bytes.as_slice()).expect("").trim_matches
            (char::from(0))
                .trim_matches('\u{0020}').to_string()))
        }
    };
}

struct FStrVisitor;
impl<'de> Visitor<'de> for FStrVisitor {
    type Value = FStr;

    fn expecting(&self, formatter: &mut Formatter) -> std::fmt::Result {
        formatter.write_str(" ")
    }
    fn visit_seq<A>(self, mut seq: A) -> Result<Self::Value, <A as SeqAccess<'de>>::Error> where
        A: SeqAccess<'de>, {
        access_fstr(&mut seq)
    }
}

impl<'de> Deserialize<'de> for FStr {
    fn deserialize<D>(deserializer: D) -> Result<Self, <D as Deserializer<'de>>::Error> where
        D: Deserializer<'de> {
        deserializer.deserialize_tuple(65540, FStrVisitor) // this is nasty but it apparently doesnt allocate!
    }
}

#[derive(Debug, Deserialize, PartialEq)]
pub struct ReplayMeta {
    pub file_version: u32,
    pub length_in_ms: u32,
    pub network_version: u32,
    pub changelist: u32,
    pub friendly_name: FStr,
}

#[derive(Debug, Deserialize, PartialEq)]
pub struct FNReplay {
    pub file_magic: u32,
    pub meta: ReplayMeta
}