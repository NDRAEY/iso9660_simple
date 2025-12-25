// https://people.freebsd.org/~emaste/rrip112.pdf

pub enum Entity<'data> {
    Name {
        name: &'data str,
    },
    PosixAttributes {
        posix_file_mode: u32,
        posix_file_links: u32,
        posix_file_user_id: u32,
        posix_file_group_id: u32,
        posix_file_serial_number: u32,
    },
}

pub struct RockRidgeParser<'data> {
    data: &'data [u8],
    index: usize
}

impl<'data> RockRidgeParser<'data> {
    pub fn new(data: &'data [u8]) -> Self {
        Self {
            data,
            index: 0,
        }
    }
}

impl<'data> Iterator for RockRidgeParser<'data> {
    type Item = Entity<'data>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.index + 2 < self.data.len() {
            let identifier = &self.data[self.index..=self.index + 1];
            let length = self.data[self.index + 2] as usize;

            // println!("ID: {:x?}; Len: {}", identifier, length);

            match identifier {
                b"SP" => {
                    // IDK what `SP` is and there's no definition in the Rock Ridge spec, so skip this entity.
                    self.index += 7;

                    return self.next();
                }
                b"PX" => {
                    // WTF???
                    // Extract data
                    // let system_use_entry_version = data[index + 4];
                    let posix_file_mode = &self.data[self.index + 5..=self.index + 12];
                    let posix_file_links = &self.data[self.index + 13..=self.index + 20];
                    let posix_file_user_id = &self.data[self.index + 21..=self.index + 28];
                    let posix_file_group_id = &self.data[self.index + 29..=self.index + 36];
                    let posix_file_serial_number = &self.data[self.index + 37..=self.index + 44];

                    // First 4 bytes are needed, because each entry here is a (LSB-MSB) pair.
                    let posix_file_mode: u32 =
                        u32::from_le_bytes(posix_file_mode[..4].try_into().unwrap());
                    let posix_file_links: u32 =
                        u32::from_le_bytes(posix_file_links[..4].try_into().unwrap());
                    let posix_file_user_id: u32 =
                        u32::from_le_bytes(posix_file_user_id[..4].try_into().unwrap());
                    let posix_file_group_id: u32 =
                        u32::from_le_bytes(posix_file_group_id[..4].try_into().unwrap());
                    let posix_file_serial_number: u32 =
                        u32::from_le_bytes(posix_file_serial_number[..4].try_into().unwrap());

                    self.index += length;
                        
                    return Some(Entity::PosixAttributes {
                        posix_file_mode,
                        posix_file_links,
                        posix_file_user_id,
                        posix_file_group_id,
                        posix_file_serial_number,
                    });
                }
                b"TF" => {
                    // let system_use_entry_version = data[index + 4];
                    // let flags = data[index + 5];

                    self.index += length;
                    return self.next();
                }
                b"CE" => {
                    // Just skip it

                    self.index += length;
                    return self.next();
                }
                b"AL" => {
                    // Do I know what the hell is this?

                    self.index += length;
                    return self.next();
                }
                b"NM" => {
                    // let system_use_entry_version = data[index + 4];
                    let flags = self.data[self.index + 4];

                    if (flags & (1 << 1)) != 0 {
                        self.index += length;

                        return Some(Entity::Name { name: "." });
                    }

                    if (flags & (1 << 2)) != 0 {
                        self.index += length;
                        
                        return Some(Entity::Name { name: ".." });
                    }

                    let start = self.index + 5;
                    let end = self.index + length;

                    let name = &self.data[start..end];

                    self.index += length;

                    return Some(Entity::Name {
                        name: unsafe { str::from_utf8_unchecked(name) },
                    });
                }
                &_ => {
                    todo!(
                        "Implement entity: {} ({}, {})",
                        str::from_utf8(identifier).unwrap(),
                        self.data[self.index],
                        self.data[self.index + 1]
                    );
                }
            }
        } else {
            None
        }
    }
}

#[inline(always)]
pub fn parse<'data>(data: &'data [u8]) -> RockRidgeParser<'data> {
    RockRidgeParser::new(data)
}
