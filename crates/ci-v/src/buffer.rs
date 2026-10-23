use crate::driver::Driver;

/// Buffer è un ...
#[derive(Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct Buffer<const N: usize = 128> {
    buffer: [u8; N],
    s: usize,
    e: usize,
}

impl<const N: usize> Default for Buffer<N> {
    fn default() -> Self {
        Self {
            buffer: [0; N],
            s: 0,
            e: 0,
        }
    }
}

impl<const N: usize> core::iter::Iterator for Buffer<N> {
    type Item = u8;

    fn next(&mut self) -> Option<Self::Item> {
        // Buffer:
        // [0,1,2,...,N-3,N-2,N-1]
        //      ^     ^
        //      s     e
        //
        // self.e è inizializzato a 0: se self.e è 0 non sono presenti byte nel
        // buffer. Inoltre nel buffer sono presenti dei byte sole se s < e.
        if self.e != 0 && self.s < self.e {
            let Some(b) = self.buffer.get(self.s) else {
                return None;
            };
            // dopo aver usato s lo incremento di 1.
            self.s += 1;

            return Some(*b);
        }

        None
    }
}

impl<const N: usize> Buffer<N> {
    pub fn with(buffer: [u8; N]) -> Self {
        Self { buffer, s: 0, e: 0 }
    }

    pub fn as_mut_slice(&mut self) -> &mut [u8] {
        self.buffer.as_mut_slice()
    }

    pub fn as_slice(&self) -> &[u8] {
        self.buffer.as_slice()
    }

    pub fn take(&mut self) -> Self {
        let mut taken = Self {
            buffer: [0; N],
            s: 0,
            e: 0,
        };

        core::mem::swap(self, &mut taken);

        taken
    }

    pub fn read<D: Driver>(&mut self, driver: &mut D) -> Result<usize, D::Error> {
        let n = driver.read(self.as_mut_slice())?;
        if n != 0 {
            self.s = 0;
            self.e = n;
        }

        Ok(n)
    }

    pub fn write<D: Driver>(&mut self, length: usize, driver: &mut D) -> Result<usize, D::Error> {
        let n = driver.write(&self.buffer[..length])?;
        Ok(n)
    }
}
