use iso9660_simple::{helpers, Read as ISORead, *};
use std::{
    fs::File,
    io::{Read, Seek, SeekFrom, Write},
};

struct FileDevice(File);

impl ISORead for FileDevice {
    fn read(&mut self, position: usize, size: usize, buffer: &mut [u8]) -> Option<()> {
        println!("Seek and read: 0x{:x}", position);

        if self.0.seek(SeekFrom::Start(position as u64)).is_err() {
            return None;
        }

        if self.0.read_exact(&mut buffer[..size]).is_ok() {
            Some(())
        } else {
            None
        }
    }
}

fn main() {
    // Get last argument in command line

    let mut args = std::env::args();

    if args.len() < 2 {
        println!("Usage: {} <filename>", args.next().unwrap());
        std::process::exit(1);
    }

    let filename = args.nth(args.len() - 1).unwrap();

    let file = File::open(filename).unwrap();
    let mut buffer = ISO9660::from_device(FileDevice(file));

    // let iso = ISO::from_raw_header(buffer);

    let data = buffer.read_root();

    // println!("{:#?}", iso);
    // let hdr = buffer.header();
    // println!("{:?}", hdr);
    // println!("{}", "=".to_string().repeat(25));
    println!("{:#?}", data);
}
