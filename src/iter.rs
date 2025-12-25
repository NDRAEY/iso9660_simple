use alloc::borrow::ToOwned;
use alloc::vec;
use core::{cell::RefCell, mem};

use crate::{
    ISO9660, ISODirectoryEntry, ISODirectoryRecord, ISOInternalFlags, PRIMARY_VOLUME_DESCRIPTOR_POSITION, Read, descriptors::{Descriptor, DescriptorType}
};

pub struct DirectoryIter<'iso> {
    iso: RefCell<&'iso mut ISO9660>,
    byte_offset: RefCell<usize>,
}

impl<'iso> DirectoryIter<'iso> {
    pub fn new(iso: &'iso mut ISO9660, byte_offset: usize) -> Self {
        Self {
            iso: iso.into(),
            byte_offset: byte_offset.into(),
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

        let main_part_size = size_of::<ISODirectoryRecord>();

        let name: String;

        if self.iso.borrow().flags.contains(ISOInternalFlags::HasJoliet) {
            name = self.iso.borrow_mut().read_joliet_name(
                *self.byte_offset.borrow() + main_part_size,
                record.file_identifier_length as _
            ).expect("Expected a valid UCS-2 name");
        } else {
            let main_part_size = main_part_size + record.file_identifier_length as usize;
            let extension_size = record.length as usize - main_part_size;

            // println!("Ext size: {}", extension_size);

            let rr_name = self.iso.borrow_mut().read_rock_ridge_name(
                *self.byte_offset.borrow(),
                main_part_size,
                extension_size,
            );

            name = if let Some(n) = rr_name {
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
        }

        *self.byte_offset.borrow_mut() += record.length as usize;

        Some(ISODirectoryEntry { record, name })
    }
}

pub struct DescriptorIterator<'dev> {
    device: &'dev mut dyn Read,
    position: usize,
}

impl<'a> DescriptorIterator<'a> {
    pub fn new(dev: &'a mut dyn Read) -> Self {
        Self {
            device: dev,
            position: PRIMARY_VOLUME_DESCRIPTOR_POSITION,
        }
    }
}

impl<'a> Iterator for DescriptorIterator<'a> {
    type Item = Descriptor;

    fn next(&mut self) -> Option<Self::Item> {
        let mut buffer = [0u8; 2048];

        self.device.read(self.position, &mut buffer)?;

        let descriptor: Self::Item = unsafe { mem::transmute(buffer) };

        if descriptor.desc_type == DescriptorType::Terminator {
            None
        } else {
            self.position += core::mem::size_of::<Descriptor>();

            Some(descriptor)
        }
    }
}
