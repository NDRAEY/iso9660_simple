// https://people.freebsd.org/~emaste/rrip112.pdf

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

        let identifier = (identifier[0] as char).to_string() + &(identifier[1] as char).to_string();

        // println!("ID: {} (Size is: {})", identifier, length);

        match identifier.as_str() {
            "SP" => {
                // IDK what `SP` is and there's no definition in the Rock Ridge spec, so skip this entity.
                index += 7;
                continue;
            }
            "PX" => {
                /// WTF???
                // Extract data
                let system_use_entry_version = data[index + 4];
                let posix_file_mode = &data[index + 5..=index + 12];
                let posix_file_links = &data[index + 13..=index + 20];
                let posix_file_user_id = &data[index + 21..=index + 28];
                let posix_file_group_id = &data[index + 29..=index + 36];
                let posix_file_serial_number = &data[index + 37..=index + 44];

                // First 4 bytes are needed, because each entry here is a (LSB-MSB) pair.
                let posix_file_mode: u32 = posix_file_mode[..4]
                    .iter()
                    .map(|x| x.to_string())
                    .collect::<String>()
                    .parse::<u32>()
                    .unwrap();
                let posix_file_links: u32 = posix_file_links[..4]
                    .iter()
                    .map(|x| x.to_string())
                    .collect::<String>()
                    .parse::<u32>()
                    .unwrap();
                let posix_file_user_id: u32 = posix_file_user_id[..4]
                    .iter()
                    .map(|x| x.to_string())
                    .collect::<String>()
                    .parse::<u32>()
                    .unwrap();
                let posix_file_group_id: u32 = posix_file_group_id[..4]
                    .iter()
                    .map(|x| x.to_string())
                    .collect::<String>()
                    .parse::<u32>()
                    .unwrap();
                let posix_file_serial_number: u32 = posix_file_serial_number[..4]
                    .iter()
                    .map(|x| x.to_string())
                    .collect::<String>()
                    .parse::<u32>()
                    .unwrap();

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
            "TF" => {
                // let system_use_entry_version = data[index + 4];
                // let flags = data[index + 5];

                index += length;
            }
            "CE" => {
                // Just skip it

                index += length;
            }
            "NM" => {
                // let system_use_entry_version = data[index + 4];
                // let flags = data[index + 5];
                let name = &data[index + 5..=index + (length - 1)];

                let rname = String::from_utf8(name.to_vec()).unwrap();

                entities.push(Entity::Name { name: rname });

                index += length;
            }
            &_ => {
                todo!(
                    "Implement entity: {} ({}, {})",
                    identifier,
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
