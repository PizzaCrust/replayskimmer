mod fnchunk;
mod uetypes;
mod uchunk;
mod skimmer;

#[macro_use] extern crate error_chain;

use skimmer::*;
use std::time::SystemTime;
use crate::uchunk::{HeaderChunk, EventChunk};
use crate::fnchunk::Elimination;

error_chain! {
    errors {
        BincodeError {
            description("bincode failure")
            display("bincode failure")
        }
        ReplayParseError(msg: String) {
            description("replay parse failure")
            display("replay parse failure: {}", msg)
        }
        EncryptionError {
            description("encryption failure")
            display("encryption failure")
        }
    }
    foreign_links {
        Io(std::io::Error);
        Time(std::time::SystemTimeError);
    }
}

macro_rules! measure {
    ($expr:expr;) => {{
           let start = SystemTime::now();
           let value = $expr;
           println!("took {} ms", SystemTime::now().duration_since(start)?.as_millis());
           value
    }};
}

fn main() -> Result<()> {
    let replay = measure! {
        UReplay::parse(std::fs::read("season12.replay")?);
    }.map_err(|e| crate::Error::with_chain(e, crate::ErrorKind::BincodeError))?;
    for x in replay.chunks {
        if x.variant == 0 {
            //println!("{:?}", bincode::deserialize::<HeaderChunk>(x.data.as_slice()))
            println!("{:?}", HeaderChunk::parse(x));
        } else if x.variant == 3 {
            let e_chunk = EventChunk::parse(x, replay.meta.encryption_key.as_slice())?;
            if e_chunk.group == "playerElim" {
                println!("{:?}", Elimination::parse(e_chunk))
            }
        }
    }
    //println!("{:?}", header_chunk);
    Ok(())
}
