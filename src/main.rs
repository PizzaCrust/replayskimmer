mod fnchunk;
mod uetypes;
mod uchunk;
mod ureplay;

#[macro_use] extern crate error_chain;

use ureplay::*;
use std::time::SystemTime;
use crate::uchunk::{HeaderChunk, EventChunk};
use crate::fnchunk::{Elimination, FNSkim};

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
        let skim = FNSkim::skim(replay)?;
        println!("{:#?}", skim);
        Ok(())
    });
    Ok(())
}
