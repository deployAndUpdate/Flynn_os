pub const PAGE_SIZE: usize = 4096;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Frame {
    pub number: usize,
}

impl Frame {
    pub fn containing_address(addr: usize) -> Self {
        Self {
            number: addr / PAGE_SIZE,
        }
    }

    pub fn start_address(self) -> usize {
        self.number * PAGE_SIZE
    }
}
