use serde::Deserialize;
//use crate::uetypes2::*;
use crate::uetypes::*;

#[derive(Debug, Deserialize, PartialEq)]
pub struct ReplayMeta {
    pub file_version: u32,
    pub length_in_ms: u32,
    pub network_version: u32,
    pub changelist: u32,
    pub friendly_name: String,
    pub is_live: bool,
    pub timestamp: u64,
    pub is_compressed: bool,
    pub is_encrypted: bool,
    pub encryption_key: Vec<u8>
}

#[derive(Debug, Deserialize, PartialEq)]
pub struct UReplay {
    pub file_magic: u32,
    pub meta: ReplayMeta
}