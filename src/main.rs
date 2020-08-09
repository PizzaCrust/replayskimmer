mod skimmer;

#[macro_use] extern crate error_chain;

use skimmer::*;

error_chain! {
    foreign_links {
        Io(std::io::Error);
    }
}

fn main() -> Result<()> {
    println!("Hello, world!");
    println!("{:?}", bincode::deserialize::<FNReplay>(std::fs::read("season12.replay")?.as_slice()));
    Ok(())
}
