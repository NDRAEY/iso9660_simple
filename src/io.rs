/// Implement reading functionality from ANYTHING by implementing this trait.
pub trait Read {
    fn read(&mut self, position: usize, size: usize, buffer: &mut [u8]) -> Option<()>;
}

// TODO: Unused?
pub trait Write {
    fn write(&mut self, position: usize, size: usize, buffer: &[u8]) -> Option<()>;
}
