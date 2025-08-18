/// Implement reading functionality from ANYTHING by implementing this trait.
pub trait Read {
    fn read(&mut self, position: usize, buffer: &mut [u8]) -> Option<()>;
}
