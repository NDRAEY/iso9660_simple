// https://people.freebsd.org/~emaste/rrip112.pdf
use alloc::vec;
use alloc::string::ToString;
use alloc::string::String;
use alloc::vec::Vec;

pub enum Entity {
    Name {
        name: String,
    },
    PosixAttributes {
        posix_file_mode: u32,
        posix_file_links: u32,
        posix_file_user_id: u32,
        posix_file_group_id: u32,
        posix_file_serial_number: u32,
    },
}

pub fn parse(data: &[u8]) -> Option<Vec<Entity>> {
    let mut index = 0;
    let mut entities: Vec<Entity> = vec![];

    while index + 2 < data.len() {
        let identifier = &data[index..=index + 1];
        let length = data[index + 2] as usize;
        // println!("ID: {} (Size is: {})", identifier, length);

        match identifier {
            b"SP" => {
                // IDK what `SP` is and there's no definition in the Rock Ridge spec, so skip this entity.
                index += 7;
                continue;
            }
            b"PX" => {
                // WTF???
                // Extract data
                // let system_use_entry_version = data[index + 4];
                let posix_file_mode = &data[index + 5..=index + 12];
                let posix_file_links = &data[index + 13..=index + 20];
                let posix_file_user_id = &data[index + 21..=index + 28];
                let posix_file_group_id = &data[index + 29..=index + 36];
                let posix_file_serial_number = &data[index + 37..=index + 44];

                // First 4 bytes are needed, because each entry here is a (LSB-MSB) pair.
                let posix_file_mode: u32 = u32::from_le_bytes(posix_file_mode[..4].try_into().unwrap());
                let posix_file_links: u32 = u32::from_le_bytes(posix_file_links[..4].try_into().unwrap());
                let posix_file_user_id: u32 = u32::from_le_bytes(posix_file_user_id[..4].try_into().unwrap());
                let posix_file_group_id: u32 = u32::from_le_bytes(posix_file_group_id[..4].try_into().unwrap());
                let posix_file_serial_number: u32 = u32::from_le_bytes(posix_file_serial_number[..4].try_into().unwrap());

                entities.push(Entity::PosixAttributes {
                    posix_file_mode,
                    posix_file_links,
                    posix_file_user_id,
                    posix_file_group_id,
                    posix_file_serial_number,
                });

                index += length;
                continue;
            }
            b"TF" => {
                // let system_use_entry_version = data[index + 4];
                // let flags = data[index + 5];

                index += length;
            }
            b"CE" => {
                // Just skip it

                index += length;
            }
            b"AL" => {
                // Do I know what the hell is this?

                index += length;
            }
            b"NM" => {
                // let system_use_entry_version = data[index + 4];
                let flags = data[index + 4];
               
                if (flags & (1 << 1)) != 0 {
                    entities.push(Entity::Name { name: String::from(".") });
                    index += length;
                    
                    continue;
                }

                if (flags & (1 << 2)) != 0 {
                    entities.push(Entity::Name { name: String::from("..") });
                    index += length;

                    continue;
                }

                let start = index + 5;
                let end = index + length;

                let name = &data[start..end];

                let name = String::from_utf8_lossy(name).to_string();

                entities.push(Entity::Name { name });

                index += length;
            }
            &_ => {
                todo!(
                    "Implement entity: {} ({}, {})",
                    String::from_utf8_lossy(identifier).to_string(),
                    data[index],
                    data[index + 1]
                );
            }
        };
    }

    if entities.is_empty() {
        None
    } else {
        Some(entities)
    }
}
