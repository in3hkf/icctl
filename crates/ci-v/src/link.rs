pub struct Frame<'a> {
    pub(crate) dst: u8,
    pub(crate) src: u8,
    pub(crate) cmd: u8,
    pub(crate) load: &'a [u8],
}

impl<'a> Frame<'a> {
    pub fn len(&self) -> usize {
        self.load.len() + 3
    }
}
