use iso9660_simple::{Read as ISORead, ISO9660};
use std::{
    fs::File,
    io::{Read, Seek, SeekFrom},
};

struct FileDevice(File);

impl ISORead for FileDevice {
    fn read(&mut self, position: usize, buffer: &mut [u8]) -> Option<()> {
        // println!("Seek and read: 0x{:x}", position);

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
    let mut buffer = ISO9660::from_device(FileDevice(file));

    // let iso = ISO::from_raw_header(buffer);

    // let data = buffer.read_root();

    // println!("{:#?}", iso);
    // let hdr = buffer.header();
    // println!("{:?}", hdr);
    // println!("{}", "=".to_string().repeat(25));
    
    fn dump(reader: &mut ISO9660, lba: u32, level: usize) {
        let data = reader.read_directory(lba as _);

        for i in data {
            let size = i.record.data_length.lsb;

            println!("{:<offset$}[{}] {} - {} bytes", "", if i.is_file() { "FILE" } else { "DIR" }, i.name, size, offset = level * 4);

            if i.is_folder() && ![".", ".."].contains(&i.name.as_str()) {
                dump(reader, i.record.lba.lsb, level + 1);
            }
        }
    }

    let root_lba = buffer.root().lba.lsb;
    dump(&mut buffer, root_lba, 0);
}
