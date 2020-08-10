use crate::ErrorKind;
use std::ptr::null;

type DecompressFunc = unsafe fn(*const u8, u64, *mut u8, u64, u32, u32, u32, u64, u64, u64, u64, u64, u64, u32) -> i32;

// https://github.com/SirWaddles/JohnWickParse/blob/master/src/decompress/oodle.rs
pub fn decompress_stream(uncompressed_size: u64, bytes: &[u8]) -> crate::Result<Vec<u8>> {
    let library = libloading::Library::new("./oo2core_7_win64.dll")?;
    let mut output = vec![0u8; uncompressed_size as usize];
    let check;
    unsafe {
        let func: libloading::Symbol<DecompressFunc> = library.get(b"OodleLZ_Decompress")?;
        check = func(bytes.as_ptr(), bytes.len() as u64, output.as_mut_ptr(), uncompressed_size, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0);
    }
    if uncompressed_size as i32 != check {
        // throw an error, work it out later
        //println!("Compression failure: {} {}", uncompressed_size, check);
        return Err(crate::ErrorKind::OodleDecodeError.into());
    }
    Ok(output)
}