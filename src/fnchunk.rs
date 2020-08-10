use crate::uchunk::EventChunk;
use std::io::{Read, Cursor};
use byteorder::ReadBytesExt;

#[derive(Debug, PartialEq)]
pub struct Elimination {
    pub victim_id: String,
    pub killer_id: String,
    pub gun_type: u8,
    pub knocked: bool
}

impl Elimination {
    fn parse_player(cursor: &mut Cursor<Vec<u8>>) -> crate::Result<String> {
        let indicator = cursor.read_u8()?;
        return Ok(match indicator {
            0x03 => {
                "Bot".to_string()
            }
            0x10 => {
                let str: String = bincode::deserialize_from(cursor).map_err(|e| crate::Error::with_chain(e, crate::ErrorKind::BincodeError))?;
                str
            }
            _ => {
                let size = cursor.read_u8()?;
                let mut guid_bytes = vec![0u8; size as usize];
                cursor.read(guid_bytes.as_mut_slice());
                hex::encode(guid_bytes)
            }
        })
    }
    pub fn parse(e: EventChunk) -> crate::Result<Elimination> {
        //e.data.as_slice();
        if e.group != "playerElim" {
            return Err(crate::ErrorKind::ReplayParseError("tried to parse another chunk as elim chunk".to_string()).into());
        }
        let mut cursor = Cursor::new(e.data);
        cursor.read(&mut [0u8; 85]);
        return Ok(Elimination {
            victim_id: Elimination::parse_player(&mut cursor)?,
            killer_id: Elimination::parse_player(&mut cursor)?,
            gun_type: cursor.read_u8()?,
            knocked: cursor.read_u32::<byteorder::LE>()? != 0
        })
    }
}