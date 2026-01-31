use iso9660_simple::helpers::get_directory_entry_by_path;
use iso9660_simple::{Read as ISORead, ISO9660};
use std::{
    fs::File,
    io::{Read, Seek, SeekFrom, Write},
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
    let mut args = std::env::args();

    if args.len() < 3 {
        println!("Usage: {} <iso-file> <path-in-iso>+", args.next().unwrap());
        std::process::exit(1);
    }

    args.next().unwrap();
    let iso_filename = args.next().unwrap();

    let file = File::open(&iso_filename).unwrap_or_else(|e| {
        eprintln!("Failed to open ISO file '{}': {}", iso_filename, e);
        std::process::exit(1);
    });

    let mut iso = match ISO9660::from_device(FileDevice(file)) {
        Some(iso) => iso,
        None => {
            eprintln!("It's not an ISO9660 (*.iso) file!");
            std::process::exit(1);
        }
    };

    for path_in_iso in args {
        let entry = match get_directory_entry_by_path(&mut iso, &path_in_iso) {
            Some(e) => e,
            None => {
                eprintln!("Path '{}' not found in ISO.", path_in_iso);
                std::process::exit(1);
            }
        };

        if entry.is_folder() {
            eprintln!("'{}' is a directory, not a file.", path_in_iso);
            std::process::exit(1);
        }

        let file_size = entry.file_size() as usize;
        let mut offset: usize = 0;
        let mut stdout = std::io::stdout();

        // Read in chunks and write to stdout
        let mut buffer = vec![0u8; 4096];

        while offset < file_size {
            let remaining = file_size - offset;
            let to_read = core::cmp::min(remaining, buffer.len());
            let buf_slice = &mut buffer[..to_read];

            if iso.read_file(&entry, offset, buf_slice).is_none() {
                eprintln!("Failed to read file data at offset {}", offset);
                std::process::exit(1);
            }

            if stdout.write_all(buf_slice).is_err() {
                eprintln!("Failed to write to stdout");
                std::process::exit(1);
            }

            offset += to_read;
        }
    }
}
