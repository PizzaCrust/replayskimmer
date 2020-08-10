use crate::skimmer::Chunk;

use serde::Deserialize;
use crate::uetypes::{GUID};

#[derive(Debug, Deserialize, PartialEq)]
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
        Ok(bincode::deserialize::<HeaderChunk>(chunk.data.as_slice()).map_err(|e| crate::Error::with_chain(e, crate::ErrorKind::BincodeError))?)
    }
}