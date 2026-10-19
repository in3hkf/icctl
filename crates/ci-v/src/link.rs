use crate::codec::{DecodeError, Decoder, State};

use embedded_io::Read;
use thiserror::Error;

#[derive(Debug)]
pub struct Frame<const N: usize> {
    pub(crate) dst: u8,
    pub(crate) src: u8,
    pub(crate) cmd: u8,
    pub(crate) load: [u8; N],
    pub(crate) load_len: usize,
}

impl<const N: usize> Default for Frame<N> {
    fn default() -> Self {
        Self {
            dst: Default::default(),
            src: Default::default(),
            cmd: Default::default(),
            load: [0; N],
            load_len: Default::default(),
        }
    }
}

impl<const N: usize> Frame<N> {
    pub fn len(&self) -> usize {
        self.load_len + 3
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
    decoder: Decoder,
    buffer: [u8; N],
    s: usize,
    e: usize,
}

impl<const N: usize> Default for Controller<N> {
    fn default() -> Self {
        Self {
            decoder: Default::default(),
            buffer: [0; N],
            s: 0,
            e: 0,
        }
    }
}

impl<const N: usize> Controller<N> {
    pub fn receive_frame<const M: usize, R: Read>(
        &mut self,
        input: &mut R,
        frame: &mut Frame<M>,
    ) -> Result<(), Error<R::Error>> {
        // [0,1,2,...,N-3,N-2,N-1]
        //      ^     ^
        //      s     e
        loop {
            // self.e è inizializzato a 0: se self.e è 0 non ci sono byte letti
            // da input nel buffer. E nel buffer ci sono byte da input se s < e
            while self.e != 0 && self.s < self.e {
                let Some(b) = self.buffer.get(self.s) else {
                    // TODO! Cambiare errore
                    return Err(Error::Internal);
                };
                // dopo aver usato s lo incremento di 1.
                self.s += 1;
                // Se decode indica che la decodifica è completa, allora in frame
                // è presente il frame decodificato ed è possiile passare il
                // controllo al chiamante.
                if self.decoder.decode(*b, frame)? == State::Complete {
                    return Ok(());
                }
            }
            // lettura da input in self.buffer... il numero di byte letti viene
            // assegnato a self.e.
            self.e = input.read(&mut self.buffer).map_err(Error::Io)?;
            // se read ritorna 0 significa che input è arrivato ad EoF.
            if self.e == 0 {
                return Err(Error::Eof);
            }
            // se read ha letto e byte allora s è 0: read inserisce a sinistra.
            self.s = 0;
        }
    }
}
