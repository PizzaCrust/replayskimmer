use std::io::Read;
use byteorder::{ReadBytesExt, LE};
use crate::uetypes::{UnrealName, UEReadExt};
use crate::ErrorKind;
use crate::strum::AsStaticRef;
use bitstream_io::LittleEndian;

// USE THIS AS MINIMALLY AS YOU CAN! REALLY SLOW!
pub struct BitReader<'a> {
    bit_size: usize,
    bit_pos: usize,
    //handle: &'a [u8],
    //bit_pos: usize,
    //last_bit: usize,
    stream: bitstream_io::BitReader<&'a mut &'a[u8], LittleEndian>
}

impl<'a> BitReader<'a> {

    pub fn new<'b>(handle: &'b mut &'b [u8], bit_size: usize) -> BitReader {
        BitReader {
            bit_size,
            bit_pos: 0,
            stream: bitstream_io::BitReader::endian(handle, LittleEndian)
        }
    }

    pub fn read_bit(&mut self) -> crate::Result<bool> {
        self.bit_pos += 1;
        Ok(self.stream.read_bit()?)
    }

    #[inline]
    pub fn remaining_len(&self) -> usize {
        self.bit_size - self.bit_pos
    }

    #[inline]
    pub fn pos(&self) -> usize {
        self.bit_pos
    }

    #[inline]
    pub fn at_end(&self) -> bool {
        self.remaining_len() <= 0
    }

    pub fn read_byte(&mut self) -> crate::Result<u8> {
        let mut byte: [u8; 1] = [0u8];
        self.read(&mut byte)?;
        self.bit_pos += 8;
        Ok(byte[0])
    }

    pub fn read_serialized_int(&mut self, max_value: u32) -> crate::Result<u32> {
        let mut value = 0u32;
        let mut mask = 1u32;
        while (value + mask) < max_value {
            if self.read_bit()? {
                value |= mask;
            }
            mask *= 2;
        }
        Ok(value)
    }

    pub fn read_bit_fname(&mut self) -> crate::Result<String> {
        let is_hardcoded = self.read_bit()?;
        if is_hardcoded {
            return Ok(UnrealName::parse(self.read_int_packed()? as i32).ok_or_else(||ErrorKind::ReplayParseError("Failed to parse fname".to_string()))?.as_static().to_string());
        }
        let in_string = self.read_fstring()?;
        let in_number = self.read_u32::<LE>()?;
        return Ok(in_string);
    }

    pub fn read_bits(&mut self, bits: &mut u32) -> crate::Result<Vec<u8>> {
        let mut vec: Vec<u8> = Vec::new();
        while *bits > 0 {
            let mut bits_to_read = *bits;
            if bits_to_read > 8 {
                bits_to_read = 8;
            }
            *bits -= bits_to_read;
            self.bit_pos += bits_to_read as usize;
            vec.push(self.stream.read::<u8>(bits_to_read)?)
        }
        Ok(vec)
    }

}

impl<'a> Read for BitReader<'a> {
    // VERY INEFFICIENT! We copy bytes from original slice!
    fn read(&mut self, buf: &mut [u8]) -> std::io::Result<usize> {
        self.stream.read_bytes(buf);
        self.bit_pos += (buf.len() * 8);
        Ok(buf.len())
    }
}