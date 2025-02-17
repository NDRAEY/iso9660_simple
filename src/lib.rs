#![no_std]

#[derive(Debug)]
#[repr(C, packed)]
pub struct ISOHeaderRaw {
    volume_descriptor_type: u8,
    id: [u8; 5],
    version: u8,
    unused00: u8,
    system_name: [u8; 32],
    label: [u8; 32],
    unused01: [u8; 8],
    volume_space_size: [u32; 2],
    un_used02: [u8; 32],
    volume_set_size: u32,
    volume_sequence_number: u32,
    logical_block_size: u32,
    path_table_size: [u32; 2],
    loc_of_type_l_path_table: u32,
    loc_of_opti_l_path_table: u32,
    loc_of_type_m_path_table: u32,
    loc_of_opti_m_path_table: u32,
    directory_entry: [u8; 34],
    volume_set_id: [u8; 128],
    publisher_id: [u8; 128],
    data_preparer_id: [u8; 128],
    application_id: [u8; 128],
    copyright_file_id: [u8; 37],
    abstract_file_id: [u8; 37],
    bibliographic_file_id: [u8; 37],
    volume_creation_date: [u8; 17],
    volume_modification_date: [u8; 17],
    volume_expiration_date: [u8; 17],
    volume_effective_date: [u8; 17],
    file_structure_version: i8,
    unused03: i8,
    application_used: [u8; 512],
    reserved: [u8; 653],
}

impl ISOHeaderRaw {
    pub fn zeroed() -> Self {
        let zeroed = [0u8; core::mem::size_of::<Self>()];

        let iso: ISOHeaderRaw = unsafe {
            core::mem::transmute(zeroed)
        };

        iso
    }

    pub fn as_slice(&mut self) -> &[u8] {
        unsafe {
            core::slice::from_raw_parts(self as *const Self as *const u8, core::mem::size_of::<Self>())
        }
    }

    pub fn as_mut_slice(&mut self) -> &mut [u8] {
        unsafe {
            core::slice::from_raw_parts_mut(self as *mut Self as *mut u8, core::mem::size_of::<Self>())
        }
    }
}

extern crate alloc;

use alloc::string::String;

#[derive(Debug)]
pub struct ISO {
    header: ISOHeaderRaw,
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
    volume_effective_date: String
}

impl ISO {
    pub fn from_raw_header(header: ISOHeaderRaw) -> Self {
        ISO {
            system_name: String::from_utf8(header.system_name.to_vec()).unwrap(),
            label: String::from_utf8(header.label.to_vec()).unwrap(),
            directory_entry: String::from_utf8(header.directory_entry.to_vec()).unwrap(),
            volume_set_id: String::from_utf8(header.volume_set_id.to_vec()).unwrap(),
            publisher_id: String::from_utf8(header.publisher_id.to_vec()).unwrap(),
            data_preparer_id: String::from_utf8(header.data_preparer_id.to_vec()).unwrap(),
            application_id: String::from_utf8(header.application_id.to_vec()).unwrap(),
            copyright_file_id: String::from_utf8(header.copyright_file_id.to_vec()).unwrap(),
            abstract_file_id: String::from_utf8(header.abstract_file_id.to_vec()).unwrap(),
            bibliographic_file_id: String::from_utf8(header.bibliographic_file_id.to_vec()).unwrap(),
            volume_creation_date: String::from_utf8(header.volume_creation_date.to_vec()).unwrap(),
            volume_modification_date: String::from_utf8(header.volume_modification_date.to_vec()).unwrap(),
            volume_expiration_date: String::from_utf8(header.volume_expiration_date.to_vec()).unwrap(),
            volume_effective_date: String::from_utf8(header.volume_effective_date.to_vec()).unwrap(),
            header
        }
    }
}

#[repr(C)]
#[derive(Debug)]
pub struct ISODateTime {
    year: u8,
    month: u8,
    day: u8,
    hour: u8,
    minute: u8,
    second: u8,
    gmt_offset: u8
}