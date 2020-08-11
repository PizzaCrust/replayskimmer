use std::time::SystemTime;
use replayskimmer::ureplay::UReplay;
use replayskimmer::fnchunk::FNSkim;
use replayskimmer::data::net::DemoFrame;

fn measure(block: fn() -> replayskimmer::Result<()>) -> replayskimmer::Result<()> {
    let current_time = SystemTime::now();
    block()?;
    println!("took {} ms", SystemTime::now().duration_since(current_time)?.as_millis());
    Ok(())
}

fn main() -> replayskimmer::Result<()> {
    measure(|| {
        let replay= UReplay::parse(std::fs::read("season12.replay")?)?;
        let skim = FNSkim::skim(replay, true)?;
        //println!("{:#?}", skim);
        //for x in skim.data_chunks.expect("") {
        //    let mut slice: &[u8] = x.data.as_slice();
        //    //println!("{:#?}", data::net::DemoFrame::parse(&mut slice)?);
        //    let mut demo_frames: Vec<DemoFrame> = Vec::new();
        //    while !slice.is_empty() {
        //        demo_frames.push(DemoFrame::parse(&mut slice)?)
        //    }
        //    //println!("{:#?}", demo_frames)
        //}
        Ok(())
    })?;
    Ok(())
}
