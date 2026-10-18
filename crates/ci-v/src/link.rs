use crate::codec::{DecodeError, Decoder, State};

use embedded_io::Read;
use thiserror::Error;

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

#[derive(Debug, Error)]
pub enum Error<E> {
    #[error("I/O error: {0:?}")]
    Io(E),
    #[error("unexpected end of stream")]
    Eof,
    #[error(transparent)]
    Decode(#[from] DecodeError),
    #[error("internal error: decoder invariant violated")]
    Internal,
}

pub struct Controller<const N: usize> {
    decoder: Decoder<N>,
}

impl<const N: usize> Controller<N> {
    pub fn receive<R: Read>(&mut self, input: &mut R) -> Result<Frame<'_>, Error<R::Error>> {
        loop {
            let mut b = [0u8; 1];
            if input.read(&mut b).map_err(Error::Io)? == 0 {
                return Err(Error::Eof);
            }
            if self.decoder.feed(b[0])? == State::Complete {
                return self.decoder.frame().ok_or(Error::Internal); // non può accadere
            }
        }
    }
}
