use std::io::Read;
use byteorder::{ReadBytesExt, LE};
use crate::uetypes::{UnrealName, UEReadExt};
use crate::ErrorKind;
use crate::strum::AsStaticRef;
use bitstream_io::LittleEndian;
use crate::data::packet::{FVector, FRotator};

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

    pub fn read_vector(&mut self) -> crate::Result<FVector> {
        Ok(FVector(self.read_f32::<LE>()?, self.read_f32::<LE>()?, self.read_f32::<LE>()?))
    }

    pub fn read_packed_vector(&mut self, scale_factor: u32, max_bits: u32) -> crate::Result<FVector> {
        let bits = self.read_serialized_int(max_bits)?;
        let bias = 1 << (bits + 1);
        let max = 1 << (bits + 2);
        let dx = self.read_serialized_int(max)?;
        let dy = self.read_serialized_int(max)?;
        let dz = self.read_serialized_int(max)?;
        let x = ((dx as f32) - (bias as f32)) / (scale_factor as f32);
        let y = ((dy as f32) - (bias as f32)) / (scale_factor as f32);
        let z = ((dz as f32) - (bias as f32)) / (scale_factor as f32);
        // ^ dont cast as f32 if want to succeed in packet vector tests :)
        Ok(FVector(x as f32, y as f32, z as f32))
    }

    pub fn read_conditionally_serialized_quantized_vector(&mut self, default_vector: FVector) -> crate::Result<FVector> {
        let b_was_serialized = self.read_bit()?;
        if b_was_serialized {
            let b_should_quantize = self.read_bit()?;
            return if b_should_quantize { self.read_packed_vector(10, 24) } else { self.read_vector() }
        }
        Ok(default_vector)
    }

    pub fn read_rotation_short(&mut self) -> crate::Result<FRotator> {
        let mut pitch: f32 = 0 as f32;
        let mut yaw: f32 = 0 as f32;
        let mut roll: f32 = 0 as f32;
        if self.read_bit()? {
            pitch = ((self.read_u16::<LE>()? as u32) * 360 / 65536) as f32;
        }
        if self.read_bit()? {
            yaw = ((self.read_u16::<LE>()? as u32) * 360 / 65536) as f32;
        }
        if self.read_bit()? {
            roll = ((self.read_u16::<LE>()? as u32) * 360 / 65536) as f32;
        }
        Ok(FRotator(pitch, yaw, roll))
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