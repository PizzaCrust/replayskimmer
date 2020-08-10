use crate::ureplay::{Chunk, UReplay};

use serde::Deserialize;
use crate::uetypes::{GUID};
use serde::export::fmt::Debug;
use serde::export::Formatter;
use std::io::Read;
use aes_soft::block_cipher::{NewBlockCipher, BlockCipher};
use aes_soft::block_cipher::generic_array::GenericArray;
use block_modes::{Ecb, BlockMode};
use aes_soft::{Aes128, Aes256};
use block_modes::block_padding::Pkcs7;
use bincode::ErrorKind;

#[derive(Debug, Deserialize, Default, PartialEq)]
pub struct HeaderChunk {
    pub network_magic: u32,
    pub network_version: u32,
    pub network_checksum: u32,
    pub engine_network_version: u32,
    pub game_network_protocol_version: u32,
    pub id: GUID,
    pub major: u16,
    pub minor: u16,
    pub patch: u16,
    pub changelist: u32,
    pub branch_data: String,
    pub level_names_and_times: Vec<(String, u32)>,
    pub flags: u32,
    pub game_specific_data: Vec<String>
}

impl HeaderChunk {
    pub fn parse(chunk: Chunk) -> crate::Result<HeaderChunk> {
        if chunk.variant != 0 {
            //panic!("tried to parse another chunk as header chunk")
            return Err(crate::ErrorKind::ReplayParseError("tried to parse another chunk as header chunk".to_string()).into());
        }
        Ok(bincode::deserialize::<HeaderChunk>(chunk.data.as_slice())?)
    }
}

#[derive(Deserialize, PartialEq)]
pub struct EventChunk {
    pub id: String,
    pub group: String,
    pub metadata: String,
    pub start_time: u32,
    pub end_time: u32,
    #[serde(with = "serde_bytes")]
    pub data: Vec<u8>
}

pub type Aes = Ecb<Aes256, Pkcs7>;

impl EventChunk {
    pub fn parse(c: Chunk, enc_key: &[u8]) -> crate::Result<EventChunk> {
        if c.variant != 3 {
            return Err(crate::ErrorKind::ReplayParseError("tried to parse another chunk as event chunk".to_string()).into());
        }
        let mut event_chunk = bincode::deserialize::<EventChunk>(c.data.as_slice())?;
        let cipher = Aes::new_var(enc_key, Default::default())?;
        event_chunk.data = cipher.decrypt_vec(event_chunk.data.as_slice())?;
        Ok(event_chunk)
    }
}

impl Debug for EventChunk {
    fn fmt(&self, f: &mut Formatter) -> std::fmt::Result {
        f.write_str(&*format!("event chunk in group {}", self.group))
    }
}