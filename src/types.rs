use zerocopy::{FromBytes, Immutable, IntoBytes};

#[repr(C, packed(1))]
#[derive(Copy, Clone, Debug, Default, FromBytes, Immutable, IntoBytes)]
pub struct LSB_MSB<T> {
    lsb: T,
    msb: T,
}

impl<T: Copy> LSB_MSB<T> {
    pub fn get(&self) -> T {
        #[cfg(target_endian = "little")]
        return self.lsb;
        #[cfg(target_endian = "big")]
        return self.msb;
    }
}
