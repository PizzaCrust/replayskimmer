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

    #[bench]
    fn parse_full_replay(b: &mut Bencher)  {
        let file = std::fs::read("season12.replay").unwrap();
        b.iter(|| {
            FNSkim::skim(UReplay::parse(file.clone()).unwrap(), true);
        });
    }

    #[test]
    fn bits() {
        //let mut reader = BitAndByteReader::new(&[0b1100]);
        //let mut reader = BitReader::new(&[0b1100]);
        let mut reader = crate::data::bitreader::BitReader::new(&[0x23], 8);
        assert_eq!(reader.read_bit(), true);
        assert_eq!(reader.read_bit(), true);
        assert_eq!(reader.read_bit(), false);
        assert_eq!(reader.read_bit(), false);
        assert_eq!(reader.read_bit(), false);
        assert_eq!(reader.read_bit(), true);
        assert_eq!(reader.read_bit(), false);
        assert_eq!(reader.read_bit(), false);
    }

    #[test]
    fn bytes() {
        let mut reader = crate::data::bitreader::BitReader::new(&[0x01, 0x02, 0x03], 24);
        assert_eq!(reader.read_byte(), 0x01);
        assert_eq!(reader.read_byte(), 0x02);
        assert_eq!(reader.read_byte(), 0x03);
    }

    #[test]
    fn int_packed() {
        let bits: [u8; 1] = [0xCC];
        let mut reader = crate::data::BitReader::new(&bits, 8);
        assert_eq!(reader.read_int_packed().expect(""), 102u32);
        assert_eq!(BitReader::new(&[0x24, 0x40], 16).read_int_packed().expect(""), 18u32);
        //println!("{}", reader.read_bit_int_packed().expect(""))
    }

    #[test]
    fn serialized_int() {
        let bits: [u8; 1] = [0x64];
        let bits2: [u8; 1] = [0x01];
        //let mut reader = BitAndByteReader::new(&bits);
        //let mut reader1 = BitAndByteReader::new(&bits2);
        //assert_eq!(reader.read_serialized_int(3).expect(""), 0u32);
        let mut reader = crate::data::BitReader::new(&bits, 8);
        let mut reader1 = crate::data::BitReader::new(&bits2, 8);
        assert_eq!(reader.read_serialized_int(3), 0u32);
        assert_eq!(reader1.read_serialized_int(2), 1u32);
    }

    #[test]
    fn fname() {
        let bits: [u8; 2] = [0x99, 0xF1];
        let mut reader = crate::data::BitReader::new(&bits, 16);
        assert_eq!(reader.read_bit_fname().expect(""), "Actor");
        assert_eq!(9, reader.pos())
    }
}