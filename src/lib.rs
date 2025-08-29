// #![no_std]

#![cfg_attr(not(feature = "std"), no_std)]

pub mod helpers;
pub mod rock_ridge;
pub mod types;
pub mod iter;

use alloc::vec;

/// Each sector in ISO is 2048 bytes (imho)
const DISK_SECTOR_SIZE: usize = 2048;

//const FLAG_HIDDEN: u8 = 1 << 0;
const FLAG_DIRECTORY: u8 = 1 << 1;
//const FLAG_ASSOCIATED: u8 = 1 << 2;
//const FLAG_EXTENDED_ATTR: u8 = 1 << 3;

/// The header that starts on offset 0x8000 (bytes) on each ISO
#[derive(Debug)]
#[repr(C, packed(1))]
pub struct ISOHeaderRaw {
    pub volume_descriptor_type: u8,
    pub id: [u8; 5],
    pub version: u8,
    pub unused00: u8,
    pub system_name: [u8; 32],
    pub label: [u8; 32],
    pub unused01: [u8; 8],
    pub volume_space_size: [u32; 2],
    pub un_used02: [u8; 32],
    pub volume_set_size: u32,
    pub volume_sequence_number: u32,
    pub logical_block_size: u32,
    pub path_table_size: [u32; 2],
    pub loc_of_type_l_path_table: u32,
    pub loc_of_opti_l_path_table: u32,
    pub loc_of_type_m_path_table: u32,
    pub loc_of_opti_m_path_table: u32,
    pub directory_entry: [u8; 34],
    pub volume_set_id: [u8; 128],
    pub publisher_id: [u8; 128],
    pub data_preparer_id: [u8; 128],
    pub application_id: [u8; 128],
    pub copyright_file_id: [u8; 37],
    pub abstract_file_id: [u8; 37],
    pub bibliographic_file_id: [u8; 37],
    pub volume_creation_date: [u8; 17],
    pub volume_modification_date: [u8; 17],
    pub volume_expiration_date: [u8; 17],
    pub volume_effective_date: [u8; 17],
    pub file_structure_version: i8,
    pub unused03: i8,
    pub application_used: [u8; 512],
    pub reserved: [u8; 653],
}

impl ISOHeaderRaw {
    /// Helper function that exposes ISO header as an array off bytes
    pub fn as_slice(&mut self) -> &[u8] {
        unsafe { core::slice::from_raw_parts(self as *const Self as *const u8, size_of::<Self>()) }
    }

    /// Helper function that exposes ISO header as a mutable array off bytes
    pub fn as_mut_slice(&mut self) -> &mut [u8] {
        unsafe { core::slice::from_raw_parts_mut(self as *mut Self as *mut u8, size_of::<Self>()) }
    }
}

extern crate alloc;

use core::mem::size_of;

use alloc::{
    borrow::ToOwned,
    boxed::Box,
    string::String,
    vec::Vec,
};

#[derive(Debug)]
pub struct ISOHeaderInfo<'this> {
    pub system_name: &'this str,
    pub label: &'this str,
    pub volume_set_id: &'this str,
    pub publisher_id: &'this str,
    pub data_preparer_id: &'this str,
    pub application_id: &'this str,
    pub copyright_file_id: &'this str,
    pub abstract_file_id: &'this str,
    pub bibliographic_file_id: &'this str,
    pub volume_creation_date: &'this str,
    pub volume_modification_date: &'this str,
    pub volume_expiration_date: &'this str,
    pub volume_effective_date: &'this str,
}

#[derive(Debug)]
pub struct ISOHeader {
    pub header: ISOHeaderRaw,
}

impl ISOHeader {
    /// Makes an ISOHeader from ISOHeaderRaw
    pub fn from_raw_header(header: ISOHeaderRaw) -> Self {
        ISOHeader { header }
    }

    pub fn info(&self) -> ISOHeaderInfo<'_> {
        let header = &self.header;

        ISOHeaderInfo {
            system_name: str::from_utf8(&header.system_name).unwrap().trim_end(),
            label: str::from_utf8(&header.label).unwrap().trim_end(),
            volume_set_id: str::from_utf8(&header.volume_set_id).unwrap().trim_end(),
            publisher_id: str::from_utf8(&header.publisher_id).unwrap().trim_end(),
            data_preparer_id: str::from_utf8(&header.data_preparer_id).unwrap().trim_end(),
            application_id: str::from_utf8(&header.application_id).unwrap().trim_end(),
            copyright_file_id: str::from_utf8(&header.copyright_file_id)
                .unwrap()
                .trim_end(),
            abstract_file_id: str::from_utf8(&header.abstract_file_id).unwrap().trim_end(),
            bibliographic_file_id: str::from_utf8(&header.bibliographic_file_id)
                .unwrap()
                .trim_end(),
            volume_creation_date: str::from_utf8(&header.volume_creation_date)
                .unwrap()
                .trim_end_matches('\0'),
            volume_modification_date: str::from_utf8(&header.volume_modification_date)
                .unwrap()
                .trim_end_matches('\0'),
            volume_expiration_date: str::from_utf8(&header.volume_expiration_date)
                .unwrap()
                .trim_end_matches('\0'),
            volume_effective_date: str::from_utf8(&header.volume_effective_date)
                .unwrap()
                .trim_end_matches('\0'),
        }
    }
}

/// Represents date and time packed into every DirectoryEntry
#[repr(C, packed(1))]
#[derive(Clone, Copy, Debug, Default)]
pub struct ISODateTime {
    pub year: u8,
    pub month: u8,
    pub day: u8,
    pub hour: u8,
    pub minute: u8,
    pub second: u8,
    pub gmt_offset: u8,
}

/// Represents a raw directory record (name is not counted in)
#[repr(C, packed(1))]
#[derive(Debug, Default, Clone)]
pub struct ISODirectoryRecord {
    pub(crate) length: u8,
    pub(crate) xar_length: u8,
    pub lba: types::LSB_MSB<u32>,
    pub data_length: types::LSB_MSB<u32>,
    pub datetime: ISODateTime,
    pub flags: u8,
    pub(crate) unit_size: u8,
    pub(crate) interleave_gap_size: u8,
    pub(crate) volume_seq_number: types::LSB_MSB<u16>,
    pub(crate) file_identifier_length: u8, // Here comes the name which size is dynamic
}

/// Represents a human-readable directory record.
#[derive(Debug, Default, Clone)]
pub struct ISODirectoryEntry {
    pub record: ISODirectoryRecord,
    pub name: String,
}

impl ISODirectoryEntry {
    /// Simple function that checks is this entry a folder
    pub const fn is_folder(&self) -> bool {
        (self.record.flags & FLAG_DIRECTORY) != 0
    }

    pub const fn is_file(&self) -> bool {
        !self.is_folder()
    }

    pub fn lsb_position(&self) -> u32 {
        self.record.lba.lsb
    }

    pub fn file_size(&self) -> u32 {
        self.record.data_length.lsb
    }
}

pub mod io;
pub use io::Read;

use crate::iter::DirectoryIter;

/// Main structure of the crate.
/// Used to read and parse data from the `device`
pub struct ISO9660 {
    data: ISOHeader,
    root_directory: ISODirectoryRecord,
    device: Box<dyn Read>,
}

impl ISO9660 {
    pub fn from_device(mut device: impl Read + 'static) -> ISO9660 {
        let mut raw_header = unsafe { core::mem::zeroed::<ISOHeaderRaw>() };

        device.read(0x8000, raw_header.as_mut_slice());

        let idr_size = core::mem::size_of::<ISODirectoryRecord>();

        let root_dir_ptr: ISODirectoryRecord = unsafe {
            (raw_header.directory_entry[..idr_size].as_ptr() as *const ISODirectoryRecord)
                .read_unaligned()
        };

        ISO9660 {
            data: ISOHeader::from_raw_header(raw_header),
            root_directory: root_dir_ptr,
            device: Box::new(device),
        }
    }

    pub fn read_rock_ridge_name(
        &mut self,
        byte_offset: usize,
        main_part_size: usize,
        extension_size: usize,
    ) -> Option<String> {
        let mut address = byte_offset + main_part_size;
        if address % 2 != 0 {
            address += 1;
        }

        let mut extension_data: Vec<u8> = vec![0; extension_size];
        self.device.read(address, &mut extension_data);

        let rock_ridge_data = rock_ridge::parse(&extension_data);

        if let Some(rr_data) = rock_ridge_data {
            for i in rr_data {
                if let rock_ridge::Entity::Name { name } = i {
                    return Some(name.to_owned());
                }
            }

            None
        } else {
            None
        }
    }

    pub fn read_directory(&mut self, start_lba: usize) -> DirectoryIter<'_> {
        let byte_offset = start_lba * DISK_SECTOR_SIZE;

        DirectoryIter::new(self, byte_offset)
    }

    pub fn read_file(
        &mut self,
        directory_entry: &ISODirectoryEntry,
        offset: usize,
        data: &mut [u8],
    ) -> Option<()> {
        if (directory_entry.record.flags & FLAG_DIRECTORY) != 0 {
            return None;
        }

        let position = directory_entry.lsb_position() as usize;
        let data_length = directory_entry.file_size() as usize;

        if offset + data.len() > data_length {
            return None;
        }

        self.device
            .read((position * DISK_SECTOR_SIZE) + offset, data);

        Some(())
    }

    #[inline]
    pub fn read_root(&mut self) -> DirectoryIter<'_> {
        self.read_directory(self.root_directory.lba.lsb as usize)
    }

    #[inline]
    pub const fn root(&self) -> &ISODirectoryRecord {
        &self.root_directory
    }

    #[inline]
    pub const fn header(&self) -> &ISOHeader {
        &self.data
    }
}
