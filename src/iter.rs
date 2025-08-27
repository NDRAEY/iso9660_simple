use core::cell::{Ref, RefCell};

use crate::{ISODirectoryEntry, ISODirectoryRecord, ISO9660};

pub struct DirectoryIter<'iso> {
    iso: RefCell<&'iso mut ISO9660>,
    byte_offset: RefCell<usize>,
}

impl<'iso> DirectoryIter<'iso> {
    pub fn new(iso: &'iso mut ISO9660, byte_offset: usize) -> Self {
        Self {
            iso: RefCell::new(iso),
            byte_offset: RefCell::new(byte_offset),
        }
    }
}

impl Iterator for &DirectoryIter<'_> {
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
            .borrow_mut()
            .device
            .read(*self.byte_offset.borrow() as _, ptr);

        if record.length == 0 {
            return None;
        }

        let main_part_size =
            size_of::<ISODirectoryRecord>() + record.file_identifier_length as usize;
        let extension_size = record.length as usize - main_part_size;

        let rr_name = ISO9660::read_rock_ridge_name(
            &mut self.iso.borrow_mut(),
            *self.byte_offset.borrow(),
            main_part_size,
            extension_size,
        );

        let name = if let Some(n) = rr_name {
            n
        } else {
            let size = record.file_identifier_length as usize;

            let mut result = vec![0; size];

            self.iso.borrow_mut().device.read(
                *self.byte_offset.borrow() + size_of::<ISODirectoryRecord>(),
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

        *self.byte_offset.borrow_mut() += record.length as usize;

        Some(ISODirectoryEntry { record, name })
    }
}

// impl Iterator for &mut DirectoryIter<'_> {
//     type Item = ISODirectoryEntry;

//     fn next(&mut self) -> Option<Self::Item> {
//         // Delegate to the DirectoryIter's next method
//         (*self).next()
//     }
// }
