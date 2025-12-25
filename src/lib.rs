#![cfg_attr(not(feature = "std"), no_std)]

pub mod descriptors;
pub mod helpers;
pub mod extensions;
pub mod types;
pub mod iter;

use alloc::vec;

/// Each sector in ISO is 2048 bytes (imho)
const DISK_SECTOR_SIZE: usize = 2048;

const PRIMARY_VOLUME_DESCRIPTOR_POSITION: usize = 0x8000;

//const FLAG_HIDDEN: u8 = 1 << 0;
const FLAG_DIRECTORY: u8 = 1 << 1;
//const FLAG_ASSOCIATED: u8 = 1 << 2;
//const FLAG_EXTENDED_ATTR: u8 = 1 << 3;

extern crate alloc;

use alloc::{
    borrow::ToOwned,
    boxed::Box,
    string::String,
    vec::Vec,
};

use bitflags::bitflags;

bitflags! {
    pub struct ISOInternalFlags: u32 {
        const HasJoliet = (1 << 0);
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

use crate::{descriptors::DescriptorType, iter::{DescriptorIterator, DirectoryIter}};

/// Main structure of the crate.
/// Used to read and parse data from the `device`
pub struct ISO9660 {
    root_directory: ISODirectoryRecord,
    flags: ISOInternalFlags,
    device: Box<dyn Read>,
}

impl ISO9660 {
    pub fn from_device(mut device: impl Read + 'static) -> Option<ISO9660> {
        let idr_size = core::mem::size_of::<ISODirectoryRecord>();
        let mut flags = ISOInternalFlags::empty();

        let pvd_desc = DescriptorIterator::new(&mut device).find(|x| x.desc_type == DescriptorType::PrimaryVolume)?;
        let mut main_descriptor = pvd_desc.try_as_pvd()?;

        let svd: Option<descriptors::Descriptor> = DescriptorIterator::new(&mut device).find(|x| x.desc_type == DescriptorType::SupplementaryVolume);

        if let Some(ref svd) = svd {
            main_descriptor = svd.try_as_svd()?;

            flags |= ISOInternalFlags::HasJoliet;

            println!("USING SVD!");
        }

        let root_dir: ISODirectoryRecord = unsafe {
            (main_descriptor.directory_entry[..idr_size].as_ptr() as *const ISODirectoryRecord)
                .read_unaligned()
        };

        Some(ISO9660 {
            root_directory: root_dir,
            flags: flags,
            device: Box::new(device),
        })
    }

    pub fn descriptors(&mut self) -> DescriptorIterator<'_> {
        DescriptorIterator::new(self.device.as_mut())
    }

    fn read_rock_ridge_name(
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

        let rock_ridge_data = extensions::rock_ridge::parse(&extension_data);

        for i in rock_ridge_data {
            if let extensions::rock_ridge::Entity::Name { name } = i {
                return Some(name.to_owned());
            }
        }

        None
    }

    fn read_joliet_name(
        &mut self,
        byte_offset: usize,
        len: usize,
    ) -> Option<String> {
        let mut address = byte_offset;
        if (address % 2 != 0) && len != 1 {
            address += 1;
        }

        let mut ucs2_name: Vec<u8> = vec![0; len];
        self.device.read(address, &mut ucs2_name);

        extensions::joliet::parse_name(&ucs2_name)
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

    // pub fn read_primary_vol_descriptor(&mut self) -> ISOHeader {
    //     let mut raw_header = unsafe { core::mem::zeroed::<ISOHeaderRaw>() };

    //     self.device.read(PRIMARY_VOLUME_DESCRIPTOR_POSITION, raw_header.as_mut_slice());

    //     ISOHeader::from_raw_header(raw_header)
    // }

    #[inline]
    pub fn read_root(&mut self) -> DirectoryIter<'_> {
        self.read_directory(self.root_directory.lba.lsb as usize)
    }

    #[inline]
    pub fn root(&self) -> &ISODirectoryRecord {
        &self.root_directory
    }
}
