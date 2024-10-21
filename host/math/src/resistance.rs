#[derive(Clone, Copy)]
pub struct Resistance {
    // ohm
    value: usize,
}

impl Resistance {
    pub fn from_ohm(value: usize) -> Self {
        Self { value }
    }

    pub fn as_ohm(&self) -> usize {
        self.value
    }
}
