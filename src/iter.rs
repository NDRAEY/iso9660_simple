use crate::{ISODirectoryEntry, ISODirectoryRecord, ISO9660};

pub struct DirectoryIter<'iso> {
    iso: &'iso mut ISO9660,
    byte_offset: usize,
}

impl<'iso> DirectoryIter<'iso> {
    pub fn new(iso: &'iso mut ISO9660, byte_offset: usize) -> Self {
        Self {
            iso,
            byte_offset,
        }
    }
}

impl Iterator for DirectoryIter<'_> {
    type Item = ISODirectoryEntry;

    fn next(&mut self) -> Option<Self::Item> {
        let mut record = ISODirectoryRecord::default();
        let ptr = unsafe {
            core::slice::from_raw_parts_mut(
                &mut record as *mut ISODirectoryRecord as *mut u8,
                size_of::<ISODirectoryRecord>(),
            )
        };

        self.iso
            .device
            .read(self.byte_offset as _, ptr);

        if record.length == 0 {
            return None;
        }

        let main_part_size =
            size_of::<ISODirectoryRecord>() + record.file_identifier_length as usize;
        let extension_size = record.length as usize - main_part_size;

        let rr_name = ISO9660::read_rock_ridge_name(
            self.iso,
            self.byte_offset,
            main_part_size,
            extension_size,
        );

        let name = if let Some(n) = rr_name {
            n
        } else {
            let size = record.file_identifier_length as usize;

            let mut result = vec![0; size];

            self.iso.device.read(
                self.byte_offset + size_of::<ISODirectoryRecord>(),
                &mut result,
            );

            let final_name = if result[0] == 0 {
                "."
            } else if result[0] == 1 {
                ".."
            } else {
                str::from_utf8(&result).unwrap()
            };

            final_name.to_owned()
        };

        self.byte_offset += record.length as usize;

        Some(ISODirectoryEntry { record, name })
    }
}
