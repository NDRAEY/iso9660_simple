use alloc::{vec::Vec};

use crate::{ISODirectoryEntry, ISO9660};

/// This helper function searches for an entry by path.
pub fn get_directory_entry_by_path(iso: &mut ISO9660, path: &str) -> Option<ISODirectoryEntry> {
    let mut stems: Vec<&str> = path.split("/").filter(|v| !v.is_empty()).collect();
    let mut entry = iso.read_root();

    if stems.is_empty() {
        return entry;
    }

    loop {
        let mut found = false;

        for i in &entry {
            if i.name == stems[0] {
                if stems.len() == 1 {
                    return Some(i.clone());
                }

                if i.is_file() {
                    return None;
                }

                entry = iso.read_directory(i.record.lba.lsb as usize);

                found = true;

                stems.remove(0);

                break;
            }
        }

        if !found {
            break;
        }
    }

    None
}
