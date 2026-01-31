use zerocopy::{FromBytes, Immutable, IntoBytes};

#[repr(C, packed(1))]
#[derive(Copy, Clone, Debug, Default, FromBytes, Immutable, IntoBytes)]
pub struct LSB_MSB<T> {
    pub lsb: T,
    pub msb: T,
}
