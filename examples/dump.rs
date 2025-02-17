use std::io::{Read, Seek, SeekFrom};

use iso9660_simple::*;

fn main() {
    // Get last argument in command line

    let mut args = std::env::args();

    if args.len() < 2 {
        println!("Usage: {} <filename>", args.next().unwrap());
        std::process::exit(1);
    }

    let filename = args.nth(args.len() - 1).unwrap();

    let mut file = std::fs::File::open(filename).unwrap();
    let mut buffer = ISOHeaderRaw::zeroed();

    file.seek(SeekFrom::Start(0x8000)).unwrap();
    file.read(buffer.as_mut_slice()).unwrap();

    // let iso = ISO::from_raw_header(buffer);

    // println!("{:#?}", iso);
    println!("{:?}", buffer);
}