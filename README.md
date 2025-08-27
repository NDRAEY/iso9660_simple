# ISO9660 Simple

It's a Rust crate that provides minimal ISO9660 images reading and parsing functionality.

# To Do

- [x] Rock Ridge extension support
- [ ] Joliet extension support

# Usage

Firstly, add `iso9660_simple` to your project:

```bash
cargo add iso9660_simple
```

Then, implement reading device trait for your device (it may be a real device, or just a file).

The complete implementation of reading device trait for file looks like this:

```rust
use std::fs::File;
use iso9660_simple::Read as ISORead;

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
```

Then, you're ready to open the device and make ISO9660 reader from it.
For example:

```rust
let device = FileDevice(File::open("image.iso").unwrap());
let mut iso = ISO9660::from_device(device);
```

And now, you can do parse an ISO9660 file:

```rust
let root_directory_lba = iso.root().lba.lsb;
let data = iso.read_directory(root_directory_lba);  // Read root directory

let first_file = data.first().unwrap();  // Get first file info

let data = iso.read_file(fist_file);  // Read the whole file into Vec<u8>.
```