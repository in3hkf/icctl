use crate::driver::Driver;

use core::convert::Infallible;
use embedded_io::{BufRead, ErrorType, Read, Write};

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
        Self {
            buffer,
            s: 0,
            e: buffer.len(),
        }
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

    pub fn read_from<D: Driver>(&mut self, driver: &mut D) -> Result<usize, D::Error> {
        let n = driver.read(self.as_mut_slice())?;
        if n != 0 {
            self.s = 0;
            self.e = n;
        }

        Ok(n)
    }

    pub fn write_to<D: Driver>(
        &mut self,
        length: usize,
        driver: &mut D,
    ) -> Result<usize, D::Error> {
        let n = driver.write(&self.buffer[..length])?;
        Ok(n)
    }

    /// Byte non ancora letti.
    #[inline]
    pub const fn len(&self) -> usize {
        self.e - self.s
    }

    #[inline]
    pub const fn is_empty(&self) -> bool {
        self.s == self.e
    }

    /// Spazio totale disponibile in scrittura (anche dopo compattazione).
    #[inline]
    pub const fn remaining(&self) -> usize {
        N - self.len()
    }

    /// Sposta i dati non letti all'inizio dell'array.
    fn compact(&mut self) {
        if self.s > 0 {
            self.buffer.copy_within(self.s..self.e, 0);
            self.e -= self.s;
            self.s = 0;
        }
    }

    #[inline]
    pub(crate) fn set_indexes(&mut self, s: usize, e: usize) {
        self.s = s;
        self.e = e;
    }
}

impl<const N: usize> ErrorType for Buffer<N> {
    type Error = Infallible;
}

impl<const N: usize> Read for Buffer<N> {
    fn read(&mut self, buf: &mut [u8]) -> Result<usize, Self::Error> {
        let n = buf.len().min(self.len());
        buf[..n].copy_from_slice(&self.buffer[self.s..self.s + n]);
        self.s += n;
        if self.s == self.e {
            // Buffer svuotato: riparto dall'inizio, evitando compattazioni future.
            self.s = 0;
            self.e = 0;
        }
        Ok(n)
    }
}

impl<const N: usize> BufRead for Buffer<N> {
    fn fill_buf(&mut self) -> Result<&[u8], Self::Error> {
        Ok(&self.buffer[self.s..self.e])
    }

    fn consume(&mut self, amt: usize) {
        self.s = (self.s + amt).min(self.e);
        if self.s == self.e {
            self.s = 0;
            self.e = 0;
        }
    }
}

impl<const N: usize> Write for Buffer<N> {
    fn write(&mut self, buf: &[u8]) -> Result<usize, Self::Error> {
        // Compatto solo se serve: lo spazio in coda non basta per tutto `buf`.
        if N - self.e < buf.len() {
            self.compact();
        }
        let n = buf.len().min(N - self.e);
        self.buffer[self.e..self.e + n].copy_from_slice(&buf[..n]);
        self.e += n;
        Ok(n)
    }

    fn flush(&mut self) -> Result<(), Self::Error> {
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn buffer_reading() {
        let mut buffer = Buffer::with([1, 2, 3, 4, 5, 6, 7, 8]);
        let mut buf = [0; 4];

        let n = buffer.read(&mut buf).unwrap();
        assert_eq!(n, 4);
        assert_eq!(buf, [1, 2, 3, 4]);

        let n = buffer.read(&mut buf).unwrap();
        assert_eq!(n, 4);
        assert_eq!(buf, [5, 6, 7, 8]);
    }
}
