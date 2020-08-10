use crate::ureplay::Chunk;
use crate::uchunk::Aes;
use block_modes::BlockMode;
use byteorder::{ReadBytesExt, LE};
use bincode::ErrorKind;
use std::io::Read;
use serde::Deserialize;
use serde::export::fmt::Debug;
use serde::export::Formatter;

mod decompress;

#[derive(Deserialize, PartialEq)]
pub struct DataChunk {
    pub start: u32,
    pub end: u32,
    pub length: u32,
    pub memory_size_in_bytes: u32,
    #[serde(skip_deserializing)]
    pub data: Vec<u8> // encrypted + compressed, decrypt then decompress (beware of extra bytes to read before decompressing!!!!!!!)
}

impl Debug for DataChunk {
    fn fmt(&self, f: &mut Formatter) -> std::fmt::Result {
        f.write_str(&*format!("data chunk with size of {}", self.data.len()))
    }
}


impl DataChunk {
    pub fn parse(c: Chunk, enc_key: &[u8]) -> crate::Result<DataChunk> {
        if c.variant != 1 {
            return Err(crate::ErrorKind::ReplayParseError("Attempting to parse a different type chunk as data chunk".to_string()).into())
        }
        let mut cursor = c.data.as_slice();
        let mut c: DataChunk = bincode::deserialize_from(&mut cursor)?;
        let mut enc_bytes = vec![0u8; c.length as usize];
        cursor.read(enc_bytes.as_mut_slice());
        let cipher = Aes::new_var(enc_key, Default::default())?;
        let mut dec_bytes_vec = cipher.decrypt_vec(enc_bytes.as_slice())?;
        let mut dec_bytes = dec_bytes_vec.as_slice();
        let decompressed_size = dec_bytes.read_i32::<LE>()?;
        let compressed_size = dec_bytes.read_i32::<LE>()?;
        let mut compressed_bytes = vec![0u8; compressed_size as usize];
        dec_bytes.read(compressed_bytes.as_mut_slice());
        let decompressed = decompress::decompress_stream(decompressed_size as u64, compressed_bytes.as_slice())?;
        c.data = decompressed;
        Ok(c)
    }
}