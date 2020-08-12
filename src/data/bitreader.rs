use std::io::Read;
use byteorder::{ReadBytesExt, LE};
use crate::uetypes::{UnrealName, UEReadExt};
use crate::ErrorKind;
use crate::strum::AsStaticRef;

// USE THIS AS MINIMALLY AS YOU CAN! REALLY SLOW!
pub struct BitReader<'a> {
    handle: &'a [u8],
    bit_pos: usize,
    last_bit: usize,
}

impl<'a> BitReader<'a> {

    pub fn new(handle: &[u8], last_bit: usize) -> BitReader {
        BitReader {
            handle,
            bit_pos: 0,
            last_bit
        }
    }

    #[inline]
    pub fn pos(&self) -> usize { self.bit_pos }

    #[inline]
    pub fn byte_pos(&self) -> usize {
        self.bit_pos >> 3
    }

    pub fn read_bit(&mut self) -> bool {
        let result = (self.handle[self.byte_pos()] & (1 << (self.bit_pos & 7))) > 0;
        self.bit_pos += 1;
        result
    }

    #[inline]
    pub fn at_end(&self) -> bool {
        self.bit_pos >= self.last_bit
    }

    #[inline]
    pub fn can_read(&self, bit_count: usize) -> bool {
        self.bit_pos + bit_count <= self.last_bit
    }

    pub fn read_byte(&mut self) -> u8 {
        let bit_count_used_in_byte = self.bit_pos & 7;
        let bit_count_left_in_byte = 8 - bit_count_used_in_byte;
        let result = if bit_count_used_in_byte == 0 {
            self.handle[self.byte_pos()]
        } else {
            (self.handle[self.byte_pos()] >> bit_count_used_in_byte) |
                ((self.handle[self.byte_pos() + 1] & ((1 << bit_count_used_in_byte) - 1)) << bit_count_left_in_byte)
        };
        self.bit_pos += 8;
        result
    }

    pub fn read_serialized_int(&mut self, max_value: u32) -> u32 {
        let mut value = 0u32;
        let mut mask = 1u32;
        while (value + mask) < max_value {
            if self.read_bit() {
                value |= mask;
            }
            mask *= 2;
        }
        value
    }

    pub fn read_bit_fname(&mut self) -> crate::Result<String> {
        let is_hardcoded = self.read_bit();
        if is_hardcoded {
            return Ok(UnrealName::parse(self.read_int_packed()? as i32).ok_or_else(||ErrorKind::ReplayParseError("Failed to parse fname".to_string()))?.as_static().to_string());
        }
        let in_string = self.read_fstring()?;
        let in_number = self.read_u32::<LE>()?;
        return Ok(in_string);
    }

}

impl<'a> Read for BitReader<'a> {
    // VERY INEFFICIENT! We copy bytes from original slice!
    fn read(&mut self, buf: &mut [u8]) -> std::io::Result<usize> {
        let mut bytes_read = 0;
        for index in 0..buf.len() {
            buf[index] = self.read_byte();
            bytes_read += 1;
        }
        Ok(bytes_read as usize)
    }
}