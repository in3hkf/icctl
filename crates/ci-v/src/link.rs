use crate::codec::{self, DecodeError, Decoder, Encoder, State};

use embedded_io::{Read, Write};
use thiserror::Error;

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord)]
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

#[derive(Debug, Error, PartialEq, Eq, PartialOrd, Ord)]
pub enum Error<E> {
    #[error("I/O error: {0:?}")]
    Io(E),
    #[error("unexpected end of stream")]
    Eof,
    #[error(transparent)]
    Decode(#[from] DecodeError),
    #[error(transparent)]
    Encode(#[from] codec::Error),
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
        // Buffer:
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

    pub fn send_frame<const M: usize, W: Write>(
        &self,
        stream: &mut W,
        frame: &Frame<M>,
    ) -> Result<usize, Error<W::Error>> {
        let mut buf = [0u8; M];
        let n = Encoder::encode(&frame, &mut buf)?;
        stream.write(&buf[..n]).map_err(Error::Io)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn receive_simple_frame() {
        let io_stream = [0x32, 0x43, 0xFE, 0xFE, 0xA4, 0xE0, 0x07, 0x00, 0xFD];
        let load: [u8; 8] = [0; 8];
        let expected_frame = Frame {
            dst: 0xA4,
            src: 0xE0,
            cmd: 0x07,
            load,
            load_len: 1,
        };
        let mut frame: Frame<8> = Frame::default();
        let mut controller: Controller<16> = Controller::default();
        controller
            .receive_frame(&mut io_stream.as_slice(), &mut frame)
            .unwrap();
        assert_eq!(frame, expected_frame);
    }

    #[test]
    fn receive_simple_frame_small_buffer() {
        let io_stream = [0x32, 0x43, 0xFE, 0xFE, 0xA4, 0xE0, 0x07, 0x00, 0xFD];
        let load: [u8; 8] = [0; 8];
        let expected_frame = Frame {
            dst: 0xA4,
            src: 0xE0,
            cmd: 0x07,
            load,
            load_len: 1,
        };
        let mut frame: Frame<8> = Frame::default();
        let mut controller: Controller<4> = Controller::default();
        controller
            .receive_frame(&mut io_stream.as_slice(), &mut frame)
            .unwrap();
        assert_eq!(frame, expected_frame);
    }

    #[test]
    fn receive_double_simple_frame() {
        let io_stream = [
            0x32, 0x43, 0xFE, 0xFE, 0xA4, 0xE0, 0x07, 0x00, 0xFD, 0xFE, 0xFE, 0xE0, 0xA4, 0xFB,
            0xFD,
        ];
        let load: [u8; 8] = [0; 8];
        let expected_frame1 = Frame {
            dst: 0xA4,
            src: 0xE0,
            cmd: 0x07,
            load,
            load_len: 1,
        };
        let expected_frame2 = Frame {
            dst: 0xE0,
            src: 0xA4,
            cmd: 0xFB,
            load,
            load_len: 0,
        };
        let mut frame: Frame<8> = Frame::default();
        let mut controller: Controller<16> = Controller::default();
        controller
            .receive_frame(&mut io_stream.as_slice(), &mut frame)
            .unwrap();
        assert_eq!(frame, expected_frame1);
        controller
            .receive_frame(&mut io_stream.as_slice(), &mut frame)
            .unwrap();
        assert_eq!(frame, expected_frame2);
    }

    #[test]
    fn receive_simple_frame_long_preamble() {
        let io_stream = [
            0x32, 0x43, 0xFE, 0xFE, 0xFE, 0xFE, 0xFE, 0xFE, 0xFE, 0xFE, 0xFE, 0xFE, 0xFE, 0xFE,
            0xFE, 0xFE, 0xA4, 0xE0, 0x07, 0x00, 0xFD,
        ];
        let load: [u8; 8] = [0; 8];
        let expected_frame = Frame {
            dst: 0xA4,
            src: 0xE0,
            cmd: 0x07,
            load,
            load_len: 1,
        };
        let mut frame: Frame<8> = Frame::default();
        let mut controller: Controller<16> = Controller::default();
        controller
            .receive_frame(&mut io_stream.as_slice(), &mut frame)
            .unwrap();
        assert_eq!(frame, expected_frame);
    }

    #[test]
    fn receive_simple_frame_eof() {
        let io_stream = [0x32, 0x43, 0xFE, 0xFE, 0xA4, 0xE0, 0x07, 0x00];
        let mut frame: Frame<8> = Frame::default();
        let mut controller: Controller<16> = Controller::default();
        assert_eq!(
            controller.receive_frame(&mut io_stream.as_slice(), &mut frame),
            Err(Error::Eof)
        );
    }

    #[test]
    fn send_ng_message() {
        let frame = Frame {
            dst: 0xE0,
            src: 0xA4,
            cmd: 0xFA,
            load: [0; 8],
            load_len: 0,
        };
        let expected_stream: [u8; 8] = [0xFE, 0xFE, 0xE0, 0xA4, 0xFA, 0xFD, 0x00, 0x00];
        let mut stream: [u8; 8] = [0; 8];

        let controller: Controller<16> = Controller::default();
        let n = controller
            .send_frame(&mut stream.as_mut_slice(), &frame)
            .unwrap();
        assert_eq!(n, 6);
        assert_eq!(stream, expected_stream);
    }

    #[test]
    fn send_simple_frame() {
        // IC-705 CI-V Ref p17 Operating frequency
        // 145.225 MHz
        let load: [u8; 16] = [0, 0, 5, 0, 2, 2, 4, 5, 0, 1, 0, 0, 0, 0, 0, 0];
        let frame = Frame {
            dst: 0xA4,
            src: 0xE0,
            cmd: 0x05,
            load,
            load_len: 10,
        };
        let expected_stream: [u8; 16] = [
            0xFE, 0xFE, 0xA4, 0xE0, 0x05, 0x00, 0x00, 0x05, 0x00, 0x02, 0x02, 0x04, 0x05, 0x00,
            0x01, 0xFD,
        ];
        let mut stream: [u8; 16] = [0; 16];

        let controller: Controller<16> = Controller::default();
        let n = controller
            .send_frame(&mut stream.as_mut_slice(), &frame)
            .unwrap();
        assert_eq!(n, 16);
        assert_eq!(stream, expected_stream);
    }
}
