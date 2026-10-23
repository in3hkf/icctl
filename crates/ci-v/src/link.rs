use std::eprintln;

use crate::{
    Address,
    buffer::Buffer,
    driver::Driver,
    link::codec::{Decoder, Encoder, State},
};

use thiserror::Error;

mod codec;

/// Frame è la PDU del livello Data Link la cui rappresentazione nel protocollo
/// CI-V è la seguente:
///
/// ```text
/// +----------+----+----+---------+-----+
/// | PREAMBLE | DA | SA | PAYLOAD | EOM |
/// +----------+----+----+---------+-----+
/// 0          2    3    4        N+4  N+1
/// ```
///
/// | Campo    | Dimensione | Descrizione                |
/// |----------|------------|----------------------------|
/// | PREAMBLE | 2 Byte     | Preambolo: due byte FE     |
/// | DA       | 1 Byte     | Destination Address        |
/// | SA       | 1 Byte     | Source Address             |
/// | PAYLOAD  | N Byte     | Payload: CI-V Cn+Sc+Data   |
/// | EOM      | 1 Byte     | End Of Message: un byte FD |
#[derive(Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct Frame<const N: usize> {
    pub(crate) da: u8,
    pub(crate) sa: u8,
    pub(crate) payload: [u8; N],
    pub(crate) payload_len: usize,
}

impl<const N: usize> Default for Frame<N> {
    fn default() -> Self {
        Self {
            da: Default::default(),
            sa: Default::default(),
            payload: [0; N],
            payload_len: Default::default(),
        }
    }
}

impl<const N: usize> Frame<N> {
    /// `len` ritorna la lunghezza della trama così calcolata:
    /// `lunghezza del payload` + 2
    /// Ossia alla lunghezza del payload si sommano i due byte dovuti agli
    /// indirizzi Da e Sa.
    pub fn len(&self) -> usize {
        self.payload_len + 2
    }
}

#[derive(Debug, Error, PartialEq, Eq, PartialOrd, Ord)]
pub enum Error<D: Driver> {
    #[error("Driver I/O error: {0:?}")]
    Driver(D::Error),
    #[error("unexpected end of stream")]
    Eof,
    #[error(transparent)]
    Codec(#[from] codec::Error),
    #[error("internal error: decoder invariant violated")]
    Internal,
}

pub struct Controller<D: Driver, const N: usize = 128> {
    driver: D,
    radio: Address,
    controller: Address,
    echo_back: bool,
    rx_buffer: Buffer<N>,
    tx_buffer: Buffer<N>,
}

impl<D: Driver, const M: usize> Controller<D, M> {
    pub fn receive_frame<const N: usize>(&mut self) -> Result<Frame<N>, Error<D>> {
        let mut decoder = Decoder::default();
        let mut frame = Frame::default();
        loop {
            for b in Buffer::take(&mut self.rx_buffer) {
                eprintln!("Buffer: {b:X}");
                if decoder.decode(b, &mut frame)? == State::Complete {
                    return Ok(frame);
                }
            }

            let n = self
                .rx_buffer
                .read(&mut self.driver)
                .map_err(|e| Error::Driver(e))?;

            // se read ritorna 0 significa che driver è arrivato ad EoF.
            if n == 0 {
                return Err(Error::Eof);
            }
        }
    }

    pub fn send_frame<const N: usize>(&mut self, frame: &Frame<N>) -> Result<usize, Error<D>> {
        let n = Encoder::encode(&frame, self.tx_buffer.as_mut_slice())?;
        self.tx_buffer
            .write(n, &mut self.driver)
            .map_err(|e| Error::Driver(e))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use thiserror::Error;

    #[derive(Debug)]
    pub struct TestDriver<const N: usize>(Buffer<N>);

    #[derive(Error, Debug, PartialEq, Eq, PartialOrd, Ord)]
    pub enum TestDriverError {
        #[error("Write error")]
        Write,
    }

    impl<const N: usize> Driver for TestDriver<N> {
        type Endpoint = Option<[u8; N]>;

        type Error = TestDriverError;

        fn connect(enpoint: Self::Endpoint) -> Result<Self, Self::Error>
        where
            Self: Sized,
        {
            let buffer = match enpoint {
                None => [0; N],
                Some(buf) => buf,
            };

            Ok(TestDriver(Buffer::with(buffer)))
        }

        fn read(&mut self, buf: &mut [u8]) -> Result<usize, Self::Error> {
            use embedded_io::Read;
            let Ok(n) = self.0.as_slice().read(buf);
            Ok(n)
        }

        fn write(&mut self, buf: &[u8]) -> Result<usize, Self::Error> {
            use embedded_io::Write;
            let Ok(n) = self.0.as_mut_slice().write(buf) else {
                return Err(Self::Error::Write);
            };
            Ok(n)
        }
    }

    #[test]
    fn receive_simple_frame() {
        let io_stream = [0x32, 0x43, 0xFE, 0xFE, 0xA4, 0xE0, 0x07, 0x00, 0xFD];
        let payload: [u8; 4] = [0x07, 0x00, 0x00, 0x00];
        let expected_frame = Frame {
            da: 0xA4,
            sa: 0xE0,
            payload: payload,
            payload_len: 2,
        };
        let mut controller: Controller<TestDriver<9>> = Controller {
            driver: TestDriver::connect(Some(io_stream)).unwrap(),
            radio: Address(0xA4),
            controller: Address(0xE0),
            echo_back: false,
            rx_buffer: Buffer::default(),
            tx_buffer: Buffer::default(),
        };
        let frame = controller.receive_frame().unwrap();
        assert_eq!(frame, expected_frame);
    }

    #[test]
    fn receive_simple_frame_small_buffer() {
        let io_stream = [
            0x32, 0x43, 0xFE, 0xFE, 0xA4, 0xE0, 0x07, 0x00, 0xFD, 0x00, 0x00, 0x00,
        ];
        let payload: [u8; 4] = [0x07, 0x00, 0x00, 0x00];
        let expected_frame = Frame {
            da: 0xA4,
            sa: 0xE0,
            payload: payload,
            payload_len: 2,
        };
        let mut controller: Controller<TestDriver<12>, 4> = Controller {
            driver: TestDriver::connect(Some(io_stream)).unwrap(),
            radio: Address(0xA4),
            controller: Address(0xE0),
            echo_back: false,
            rx_buffer: Buffer::default(),
            tx_buffer: Buffer::default(),
        };
        let frame = controller.receive_frame().unwrap();
        assert_eq!(frame, expected_frame);
    }

    // #[test]
    // fn receive_double_simple_frame() {
    //     let io_stream = [
    //         0x32, 0x43, 0xFE, 0xFE, 0xA4, 0xE0, 0x07, 0x00, 0xFD, 0xFE, 0xFE, 0xE0, 0xA4, 0xFB,
    //         0xFD,
    //     ];
    //     let load: [u8; 8] = [0; 8];
    //     let expected_frame1 = Frame {
    //         dst: 0xA4,
    //         src: 0xE0,
    //         cmd: 0x07,
    //         load,
    //         load_len: 1,
    //     };
    //     let expected_frame2 = Frame {
    //         dst: 0xE0,
    //         src: 0xA4,
    //         cmd: 0xFB,
    //         load,
    //         load_len: 0,
    //     };
    //     let mut frame: Frame<8> = Frame::default();
    //     let mut controller: Controller<16> = Controller::default();
    //     controller
    //         .receive_frame(&mut io_stream.as_slice(), &mut frame)
    //         .unwrap();
    //     assert_eq!(frame, expected_frame1);
    //     controller
    //         .receive_frame(&mut io_stream.as_slice(), &mut frame)
    //         .unwrap();
    //     assert_eq!(frame, expected_frame2);
    // }

    // #[test]
    // fn receive_simple_frame_long_preamble() {
    //     let io_stream = [
    //         0x32, 0x43, 0xFE, 0xFE, 0xFE, 0xFE, 0xFE, 0xFE, 0xFE, 0xFE, 0xFE, 0xFE, 0xFE, 0xFE,
    //         0xFE, 0xFE, 0xA4, 0xE0, 0x07, 0x00, 0xFD,
    //     ];
    //     let load: [u8; 8] = [0; 8];
    //     let expected_frame = Frame {
    //         dst: 0xA4,
    //         src: 0xE0,
    //         cmd: 0x07,
    //         load,
    //         load_len: 1,
    //     };
    //     let mut frame: Frame<8> = Frame::default();
    //     let mut controller: Controller<16> = Controller::default();
    //     controller
    //         .receive_frame(&mut io_stream.as_slice(), &mut frame)
    //         .unwrap();
    //     assert_eq!(frame, expected_frame);
    // }

    // #[test]
    // fn receive_simple_frame_eof() {
    //     let io_stream = [0x32, 0x43, 0xFE, 0xFE, 0xA4, 0xE0, 0x07, 0x00];
    //     let mut frame: Frame<8> = Frame::default();
    //     let mut controller: Controller<16> = Controller::default();
    //     assert_eq!(
    //         controller.receive_frame(&mut io_stream.as_slice(), &mut frame),
    //         Err(Error::Eof)
    //     );
    // }

    // #[test]
    // fn send_ng_message() {
    //     let frame = Frame {
    //         dst: 0xE0,
    //         src: 0xA4,
    //         cmd: 0xFA,
    //         load: [0; 8],
    //         load_len: 0,
    //     };
    //     let expected_stream: [u8; 8] = [0xFE, 0xFE, 0xE0, 0xA4, 0xFA, 0xFD, 0x00, 0x00];
    //     let mut stream: [u8; 8] = [0; 8];

    //     let controller: Controller<16> = Controller::default();
    //     let n = controller
    //         .send_frame(&mut stream.as_mut_slice(), &frame)
    //         .unwrap();
    //     assert_eq!(n, 6);
    //     assert_eq!(stream, expected_stream);
    // }

    // #[test]
    // fn send_simple_frame() {
    //     // IC-705 CI-V Ref p17 Operating frequency
    //     // 145.225 MHz
    //     let load: [u8; 16] = [0, 0, 5, 0, 2, 2, 4, 5, 0, 1, 0, 0, 0, 0, 0, 0];
    //     let frame = Frame {
    //         dst: 0xA4,
    //         src: 0xE0,
    //         cmd: 0x05,
    //         load,
    //         load_len: 10,
    //     };
    //     let expected_stream: [u8; 16] = [
    //         0xFE, 0xFE, 0xA4, 0xE0, 0x05, 0x00, 0x00, 0x05, 0x00, 0x02, 0x02, 0x04, 0x05, 0x00,
    //         0x01, 0xFD,
    //     ];
    //     let mut stream: [u8; 16] = [0; 16];

    //     let controller: Controller<16> = Controller::default();
    //     let n = controller
    //         .send_frame(&mut stream.as_mut_slice(), &frame)
    //         .unwrap();
    //     assert_eq!(n, 16);
    //     assert_eq!(stream, expected_stream);
    // }
}
