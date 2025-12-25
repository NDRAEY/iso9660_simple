#[repr(u8)]
#[derive(Debug, PartialEq, Eq)]
pub enum DescriptorType {
    BootRecord = 0x00,
    PrimaryVolume = 0x01,
    SupplementaryVolume = 0x02,
    VolumePartition = 0x03,
    Terminator = 0xff,
}

#[repr(packed(1))]
pub struct Descriptor {
    pub desc_type: DescriptorType,
    pub id: [u8; 5],
    pub version: u8,
    pub data: [u8; 2041],
}

impl Descriptor {
    pub fn try_as_pvd(&self) -> Option<&PrimarySupplementaryVolumeDescriptor> {
        if self.desc_type == DescriptorType::PrimaryVolume {
            unsafe { (self.data.as_ptr() as *const PrimarySupplementaryVolumeDescriptor).as_ref() }
        } else {
            None
        }
    }
    
    pub fn try_as_svd(&self) -> Option<&PrimarySupplementaryVolumeDescriptor> {
        if self.desc_type == DescriptorType::SupplementaryVolume {
            unsafe { (self.data.as_ptr() as *const PrimarySupplementaryVolumeDescriptor).as_ref() }
        } else {
            None
        }
    }
}

#[derive(Debug)]
#[repr(C, packed(1))]
pub struct PrimarySupplementaryVolumeDescriptor {
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

impl PrimarySupplementaryVolumeDescriptor {
    /// Helper function that exposes ISO header as an array of bytes
    pub fn as_slice(&mut self) -> &[u8] {
        unsafe { core::slice::from_raw_parts(self as *const Self as *const u8, size_of::<Self>()) }
    }

    /// Helper function that exposes ISO header as a mutable array of bytes
    pub fn as_mut_slice(&mut self) -> &mut [u8] {
        unsafe { core::slice::from_raw_parts_mut(self as *mut Self as *mut u8, size_of::<Self>()) }
    }
}
