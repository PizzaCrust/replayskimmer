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

fn measure(block: fn() -> Result<()>) -> Result<()> {
    let current_time = SystemTime::now();
    block();
    println!("took {} ms", SystemTime::now().duration_since(current_time)?.as_millis());
    Ok(())
}

fn main() -> Result<()> {
    measure(|| {
        let replay= UReplay::parse(std::fs::read("season12.replay")?)?;
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
        Ok(())
    });
    Ok(())
}
