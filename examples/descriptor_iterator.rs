use iso9660_simple::{Read as ISORead, ISO9660};
use std::{
    fs::File,
    io::{Read, Seek, SeekFrom},
};

struct FileDevice(File);

impl ISORead for FileDevice {
    fn read(&mut self, position: usize, buffer: &mut [u8]) -> Option<()> {
        if self.0.seek(SeekFrom::Start(position as u64)).is_err() {
            return None;
        }

        if self.0.read_exact(buffer).is_ok() {
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
    let mut iso = match ISO9660::from_device(FileDevice(file)) {
        Some(iso) => iso,
        None => {
            eprintln!("It's not an ISO9660 (*.iso) file!");
            std::process::exit(1);
        },
    };

    for (n, i) in iso.descriptors().enumerate() {
        println!("Descriptor #{n:02x}; Type: {:?}; ", i.desc_type);
    }
}
