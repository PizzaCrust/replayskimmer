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
        let mut reader = crate::data::bitreader::BitReader::new(&mut bytes);
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
        let mut reader = crate::data::bitreader::BitReader::new(&mut bytes);
        assert_eq!(reader.read_byte().unwrap(), 0x01);
        assert_eq!(reader.read_byte().unwrap(), 0x02);
        assert_eq!(reader.read_byte().unwrap(), 0x03);
    }

    #[test]
    fn int_packed() {
        let mut bits: &[u8] = &[0xCC];
        let mut reader = crate::data::BitReader::new(&mut bits);
        assert_eq!(reader.read_int_packed().expect(""), 102u32);
        let mut bytes: &[u8] = &[0x24, 0x40];
        assert_eq!(BitReader::new(&mut bytes).read_int_packed().expect(""), 18u32);
    }

    #[test]
    fn serialized_int() {
        let mut bits: &[u8] = &[0x64];
        let mut bits2: &[u8] = &[0x01];
        let mut reader = crate::data::BitReader::new(&mut (bits));
        let mut reader1 = crate::data::BitReader::new(&mut (bits2));
        assert_eq!(reader.read_serialized_int(3).unwrap(), 0u32);
        assert_eq!(reader1.read_serialized_int(2).unwrap(), 1u32);
    }

    #[test]
    fn fname() {
        let mut bits: &[u8] = &[0x99, 0xF1];
        let mut reader = crate::data::BitReader::new(&mut (bits));
        assert_eq!(reader.read_bit_fname().expect(""), "Actor");
        assert_eq!(9, reader.pos())
    }

    #[test]
    fn bits() {
        let mut cursor: &[u8] = &[0x23, 0x01];
        let mut reader = BitReader::new(&mut cursor);
        assert_eq!(reader.read_bits(&mut 7u32).expect(""), vec![0x23]);
        assert_eq!(reader.remaining_len(), 9);
        reader.read_bit();
        assert_eq!(reader.read_byte().unwrap(), 0x01);
    }
}