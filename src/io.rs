pub trait Read {
    fn read(&mut self, position: usize, size: usize, buffer: &mut [u8]) -> Option<()>;
}

pub trait Write {
    fn write(&mut self, position: usize, size: usize, buffer: &[u8]) -> Option<()>;
}