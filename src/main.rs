mod skimmer;

#[macro_use] extern crate error_chain;

use skimmer::*;
use std::time::SystemTime;

error_chain! {
    errors {
        BincodeError {
            description("bincode failure")
            display("bincode failure")
        }
    }
    foreign_links {
        Io(std::io::Error);
        Time(std::time::SystemTimeError);
    }
}

macro_rules! measure {
    ($($expr:expr;),*) => {
           let start = SystemTime::now();
           {
               $($expr)*
           }
           println!("took {} ms", SystemTime::now().duration_since(start)?.as_millis());
    };
}

fn main() -> Result<()> {
    measure! {
        //println!("{:?}", bincode::deserialize::<UReplay>(std::fs::read("season12.replay")?.as_slice()));
        println!("{:?}", UReplay::parse(std::fs::read("season12.replay")?));
    }
    Ok(())
}
