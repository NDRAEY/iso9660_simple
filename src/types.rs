#[repr(packed(1))]
#[derive(Copy, Clone, Debug, Default)]
pub struct LSB_MSB<T> {
    pub lsb: T,
    pub msb: T,
}
