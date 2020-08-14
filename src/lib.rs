#![feature(test)]

#[cfg(target_os = "windows")]
pub mod data;
pub mod fnchunk;
pub mod uetypes;
pub mod uchunk;
pub mod ureplay;

extern crate strum;
#[macro_use] extern crate error_chain;
#[macro_use] extern crate strum_macros;

error_chain! {
    errors {
        ReplayParseError(msg: String) {
            description("replay parse failure")
            display("replay parse failure: {}", msg)
        }
        OodleDecodeError {
            description("oodle decode failure")
            display("oodle decode failure")
        }
    }
    foreign_links {
        Bincode(bincode::Error);
        Io(std::io::Error);
        Time(std::time::SystemTimeError);
        Enc(block_modes::BlockModeError);
        Iv(block_modes::InvalidKeyIvLength);
        Native(libloading::Error);
    }
}

#[cfg(test)]
mod tests {
    extern crate test;
    use crate::fnchunk::FNSkim;
    use crate::ureplay::UReplay;
    use test::Bencher;
    use crate::uetypes::UEReadExt;
    use crate::data::BitReader;
    use bitstream_io::LittleEndian;
    use std::io::Read;
    use crate::data::packet::FVector;

    #[bench]
    fn parse_full_replay(b: &mut Bencher)  {
        let file = std::fs::read("season12.replay").unwrap();
        b.iter(|| {
            FNSkim::skim(UReplay::parse(file.clone()).unwrap(), true);
        });
    }

    #[test]
    fn bit() {
        //let mut reader = BitAndByteReader::new(&[0b1100]);
        //let mut reader = BitReader::new(&[0b1100]);
        let mut bytes: &[u8] = &[0x23];
        let mut reader = crate::data::bitreader::BitReader::new(&mut bytes, 8);
        assert_eq!(reader.read_bit().unwrap(), true);
        assert_eq!(reader.read_bit().unwrap(), true);
        assert_eq!(reader.read_bit().unwrap(), false);
        assert_eq!(reader.read_bit().unwrap(), false);
        assert_eq!(reader.read_bit().unwrap(), false);
        assert_eq!(reader.read_bit().unwrap(), true);
        assert_eq!(reader.read_bit().unwrap(), false);
        assert_eq!(reader.read_bit().unwrap(), false);
    }

    #[test]
    fn bytes() {
        let mut bytes: &[u8] = &[0x01, 0x02, 0x03];
        let mut reader = crate::data::bitreader::BitReader::new(&mut bytes, 24);
        assert_eq!(reader.read_byte().unwrap(), 0x01);
        assert_eq!(reader.read_byte().unwrap(), 0x02);
        assert_eq!(reader.read_byte().unwrap(), 0x03);
    }

    #[test]
    fn int_packed() {
        let mut bits: &[u8] = &[0xCC];
        let mut reader = crate::data::BitReader::new(&mut bits, 8);
        assert_eq!(reader.read_int_packed().expect(""), 102u32);
        let mut bytes: &[u8] = &[0x24, 0x40];
        assert_eq!(BitReader::new(&mut bytes, 16).read_int_packed().expect(""), 18u32);
    }

    #[test]
    fn serialized_int() {
        let mut bits: &[u8] = &[0x64];
        let mut bits2: &[u8] = &[0x01];
        let mut reader = crate::data::BitReader::new(&mut (bits), 8);
        let mut reader1 = crate::data::BitReader::new(&mut (bits2), 8);
        assert_eq!(reader.read_serialized_int(3).unwrap(), 0u32);
        assert_eq!(reader1.read_serialized_int(2).unwrap(), 1u32);
    }

    #[test]
    fn fname() {
        let mut bits: &[u8] = &[0x99, 0xF1];
        let mut reader = crate::data::BitReader::new(&mut (bits), 16);
        assert_eq!(reader.read_bit_fname().expect(""), "Actor");
        assert_eq!(9, reader.pos())
    }

    #[test]
    fn bits() {
        let mut cursor: &[u8] = &[0x23, 0x01];
        let mut reader = BitReader::new(&mut cursor, 16);
        assert_eq!(reader.read_bits(&mut 7u32).expect(""), vec![0x23]);
        assert_eq!(reader.remaining_len(), 9);
        reader.read_bit();
        assert_eq!(reader.read_byte().unwrap(), 0x01);
    }

    #[test]
    fn fvector() {
        let mut cursor: &[u8] = &[0x70, 0x99, 0x7F, 0x3F, 0x00, 0x00, 0x80, 0x3F, 0x00, 0x00, 0x80, 0x3F];
        let mut cursor2: &[u8] = &[0xD3, 0x89, 0x7F, 0x3F, 0xBB, 0x08, 0x80, 0x3F, 0x00, 0x00, 0x80, 0x3F];
        let mut b1 = BitReader::new(&mut cursor, 12 * 8);
        let mut b2 = BitReader::new(&mut cursor2, 12 * 8);
        assert_eq!(b1.read_vector().unwrap(), FVector(0.998435020446777, 1 as f32, 1 as f32));
        assert_eq!(b2.read_vector().unwrap(), FVector(0.99819678068161, 1.00026643276215, 1 as f32));
    }

    #[test]
    fn packed_vector() {
        let mut a_bytes: &[u8] = &[0xB4, 0xC5, 0x5C, 0xEF, 0x81, 0x33, 0x76, 0x33, 0x3F];
        let mut b_bytes: &[u8] = &[0x74, 0xF3, 0x74, 0xC7, 0xB4, 0x2D, 0x62, 0x51, 0x3F];
        let mut c_bytes: &[u8] = &[0x98, 0xE4, 0x52, 0x62, 0x07, 0x9A, 0x75, 0x70, 0x4F, 0xF9, 0x03];
        let mut d_bytes: &[u8] = &[0x98, 0x5A, 0xF6, 0x63, 0x8C, 0x4B, 0x7A, 0x46, 0x08, 0xF8, 0x03];
        let mut e_bytes: &[u8] = &[0x40, 0x05];
        let a_bytes_len = a_bytes.len() * 8;
        let b_bytes_len = b_bytes.len() * 8;
        let c_bytes_len = c_bytes.len() * 8;
        let d_bytes_len = d_bytes.len() * 8;
        let e_bytes_len = e_bytes.len() * 8;
        let mut a: BitReader = BitReader::new(&mut a_bytes, a_bytes_len);
        let mut b: BitReader = BitReader::new(&mut b_bytes, b_bytes_len);
        let mut c: BitReader = BitReader::new(&mut c_bytes, c_bytes_len);
        let mut d: BitReader = BitReader::new(&mut d_bytes, d_bytes_len);
        let mut e: BitReader = BitReader::new(&mut e_bytes, e_bytes_len);
        assert_eq!(a.read_packed_vector(10, 24).unwrap(), FVector(176286 as f32, -167520 as f32, -2618 as f32));
        assert_eq!(b.read_packed_vector(10, 24).unwrap(), FVector(181237 as f32, -172272 as f32, -2235 as f32));
        assert_eq!(c.read_packed_vector(100, 30).unwrap(), FVector(179955 as f32, -181401 as f32, -2192 as f32));
        assert_eq!(d.read_packed_vector(100, 30).unwrap(), FVector(188546 as f32, -175249 as f32, -2610 as f32));
        assert_eq!(e.read_packed_vector(1, 24).unwrap(), FVector(0 as f32, 0 as f32, 0 as f32))
    }
}