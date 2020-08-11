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

    #[bench]
    fn parse_full_replay(b: &mut Bencher)  {
        let file = std::fs::read("season12.replay").unwrap();
        b.iter(|| {
            FNSkim::skim(UReplay::parse(file.clone()).unwrap(), true);
        });
    }
}