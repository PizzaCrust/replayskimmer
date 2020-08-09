use serde::Deserialize;
use serde::export::fmt::Debug;
use serde::export::Formatter;

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
    #[serde(with = "serde_bytes")]
    pub encryption_key: Vec<u8>
}

#[derive(Debug, Deserialize, PartialEq)]
pub struct UReplay {
    pub file_magic: u32,
    pub meta: ReplayMeta,
    #[serde(skip_deserializing)]
    pub chunks: Vec<Chunk>
}

#[derive(Deserialize, PartialEq)]
pub struct Chunk {
    variant: u32,
    #[serde(with = "serde_bytes")]
    data: Vec<u8>
}

impl Debug for Chunk {
    fn fmt(&self, f: &mut Formatter) -> std::fmt::Result {
        f.write_str(&*format!("chunk type {}", self.variant))
    }
}

impl UReplay {
    pub fn parse(bytes: Vec<u8>) -> crate::Result<UReplay> {
        let mut slice = bytes.as_slice();
        let mut replay: UReplay = bincode::deserialize_from(&mut slice).map_err(|e| crate::Error::with_chain(e, crate::ErrorKind::BincodeError))?;
        while !slice.is_empty() {
            replay.chunks.push(bincode::deserialize_from(&mut slice).map_err(|e| crate::Error::with_chain(e, crate::ErrorKind::BincodeError))?);
        }
        Ok(replay)
    }
}