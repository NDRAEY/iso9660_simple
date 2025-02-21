#![no_std]

pub mod types;

const DISK_SECTOR_SIZE: usize = 2048;

#[derive(Debug)]
#[repr(C, packed)]
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
    pub fn zeroed() -> Self {
        let zeroed = [0u8; size_of::<Self>()];

        let iso: ISOHeaderRaw = unsafe { core::mem::transmute(zeroed) };

        iso
    }

    pub fn as_slice(&mut self) -> &[u8] {
        unsafe { core::slice::from_raw_parts(self as *const Self as *const u8, size_of::<Self>()) }
    }

    pub fn as_mut_slice(&mut self) -> &mut [u8] {
        unsafe { core::slice::from_raw_parts_mut(self as *mut Self as *mut u8, size_of::<Self>()) }
    }
}

extern crate alloc;

use core::mem::{size_of, transmute_copy};

use alloc::{boxed::Box, string::{String, ToString}, vec::Vec};

#[derive(Debug)]
pub struct ISOHeader {
    pub(crate) header: ISOHeaderRaw,
    system_name: String,
    label: String,
    directory_entry: String,
    volume_set_id: String,
    publisher_id: String,
    data_preparer_id: String,
    application_id: String,
    copyright_file_id: String,
    abstract_file_id: String,
    bibliographic_file_id: String,
    volume_creation_date: String,
    volume_modification_date: String,
    volume_expiration_date: String,
    volume_effective_date: String,
}

impl ISOHeader {
    pub fn from_raw_header(header: ISOHeaderRaw) -> Self {
        ISOHeader {
            system_name: String::from_utf8(header.system_name.to_vec()).unwrap(),
            label: String::from_utf8(header.label.to_vec()).unwrap(),
            directory_entry: String::from_utf8(header.directory_entry.to_vec()).unwrap(),
            volume_set_id: String::from_utf8(header.volume_set_id.to_vec()).unwrap(),
            publisher_id: String::from_utf8(header.publisher_id.to_vec()).unwrap(),
            data_preparer_id: String::from_utf8(header.data_preparer_id.to_vec()).unwrap(),
            application_id: String::from_utf8(header.application_id.to_vec()).unwrap(),
            copyright_file_id: String::from_utf8(header.copyright_file_id.to_vec()).unwrap(),
            abstract_file_id: String::from_utf8(header.abstract_file_id.to_vec()).unwrap(),
            bibliographic_file_id: String::from_utf8(header.bibliographic_file_id.to_vec())
                .unwrap(),
            volume_creation_date: String::from_utf8(header.volume_creation_date.to_vec()).unwrap(),
            volume_modification_date: String::from_utf8(header.volume_modification_date.to_vec())
                .unwrap(),
            volume_expiration_date: String::from_utf8(header.volume_expiration_date.to_vec())
                .unwrap(),
            volume_effective_date: String::from_utf8(header.volume_effective_date.to_vec())
                .unwrap(),
            header,
        }
    }
}

#[repr(packed(1))]
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

#[repr(packed(1))]
#[derive(Debug, Default)]
pub struct ISODirectoryRecord {
    pub(crate) length: u8,
    pub(crate) xar_length: u8,
    pub lba: types::LSB_MSB<u32>,
    pub(crate) data_length: types::LSB_MSB<u32>,
    pub(crate) datetime: ISODateTime,
    pub(crate) flags: u8,
    pub(crate) unit_size: u8,
    pub(crate) interleave_gap_size: u8,
    pub(crate) volume_seq_number: types::LSB_MSB<u16>,
    pub(crate) file_identifier_length: u8, // Here comes the name which size is dynamic
}

#[derive(Debug, Default)]
pub struct ISODirectory {
    pub(crate) record: ISODirectoryRecord,
    pub name: String,
}

pub mod io;
pub use io::{Read, Write};

pub struct ISO9660 {
    data: ISOHeader,
    device: Box<dyn Read>,
}

impl ISO9660 {
    pub fn from_device(mut device: impl Read + 'static) -> ISO9660 {
        let mut raw_header = ISOHeaderRaw::zeroed();
        let read_size = size_of::<ISOHeaderRaw>();

        device.read(0x8000, read_size, raw_header.as_mut_slice());

        ISO9660 {
            data: ISOHeader::from_raw_header(raw_header),
            device: Box::new(device),
        }
    }

    pub fn read_directory(&mut self, start_lba: usize) -> Vec<ISODirectory> {
        let mut result = Vec::<ISODirectory>::new();

        let mut byte_offset = start_lba * DISK_SECTOR_SIZE;
        let mut end = false;

        while !end {
            let mut record = ISODirectoryRecord::default();
            let ptr = unsafe {
                core::slice::from_raw_parts_mut(
                    &mut record as *mut ISODirectoryRecord as *mut u8,
                    size_of::<ISODirectoryRecord>(),
                )
            };

            self.device
                .read(byte_offset as _, size_of::<ISODirectoryRecord>(), ptr);

            if record.length == 0 {
                end = true;
                break;
            }

            let mut name = {
                let size = record.file_identifier_length as usize;
                let mut result = Vec::with_capacity(size);
                unsafe { result.set_len(size); }

                self.device.read(
                    byte_offset + size_of::<ISODirectoryRecord>(),
                    size,
                    result.as_mut_slice()
                );

                String::from_utf8_lossy(result.as_slice()).to_string()
            };

            if name == "\0" {
                name = ".".to_string();
            }

            if name == "\u{1}" {
                name = "..".to_string();
            }

            byte_offset += record.length as usize;

            result.push(ISODirectory {
                record,
                name,
            });
        }

        result
    }

    pub fn read_root(&mut self) -> Vec<ISODirectory> {
        let root_dir_ptr: ISODirectoryRecord =
            unsafe { transmute_copy(&self.data.header.directory_entry) };

        self.read_directory(root_dir_ptr.lba.lsb as usize)
    }

    pub fn header(&self) -> &ISOHeader {
        &self.data
    }
}
