use core::fmt::Display;

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
    pub da: u8,
    pub sa: u8,
    pub payload: [u8; N],
    pub payload_len: usize,
}

impl<const N: usize> Display for Frame<N> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        writeln!(f, "--- Frame --------------------")?;
        match self.sa {
            0xE0 => writeln!(f, "Request"),
            _ if self.da == 0xE0 => writeln!(f, "Response"),
            _ => writeln!(f, "Req or Res"),
        }?;
        writeln!(f, "   {:X} -> {:X}", self.sa, self.da)?;
        writeln!(f, "   Payload [{} bytes]", self.payload_len)?;
        match self.payload_len {
            0 => write!(f, "   No Cn?!"),
            1 => match self.payload[0] {
                0xFA => writeln!(f, "   FA -> NG"),
                0xFB => writeln!(f, "   FB -> OK"),
                n @ _ => writeln!(f, "   Cn: {n:X}"),
            },
            2 => writeln!(
                f,
                "   Cn: {:X}\n   Data: [{:X}]",
                self.payload[0], self.payload[1]
            ),
            3 => writeln!(
                f,
                "      Cn: {:X}\n   Data: [{:X}, {:X}]",
                self.payload[0], self.payload[1], self.payload[2]
            ),
            n @ _ => {
                writeln!(f, "        Cn: {:X}", self.payload[0])?;
                write!(f, "      Data: [")?;
                for b in self.payload[1..self.payload_len - 2].iter() {
                    write!(f, "{b:X}, ")?;
                }
                writeln!(f, "{:X}]", self.payload[self.payload_len - 1])
            }
        }?;

        write!(f, "------------------------------")
    }
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

/// Link
pub trait Link {
    type Error;

    /// receive_frame riceve un singolo Frame dal link stabilito con la radio.
    fn receive_frame<const N: usize>(&mut self) -> Result<Frame<N>, Self::Error>;

    /// send_frame invia sul link il Frame fornito come argomento.
    fn send_frame<const N: usize>(&mut self, frame: &Frame<N>) -> Result<usize, Self::Error>;
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
    // radio, controller e echo_back non vengono usati in controller.
    // L'interfaccia prevede di usare Frame che ha i campi Da e Sa che con i
    // campi radio e controller potrebbero permettere un controllo di flusso
    // a livello di data link. Valutare la possibile estensione di Controller
    // in tal senso... TODO!
    radio: Address,
    controller: Address,
    echo_back: bool,
    rx_buffer: Buffer<N>,
    tx_buffer: Buffer<N>,
    decoder: Decoder,
}

impl<D: Driver, const M: usize> Controller<D, M> {
    pub fn with(driver: D) -> Self {
        Self {
            driver,
            radio: Address(0xA4),
            controller: Address(0xE0),
            echo_back: false,
            rx_buffer: Buffer::default(),
            tx_buffer: Buffer::default(),
            decoder: Decoder::default(),
        }
    }
}

impl<D: Driver, const M: usize> Link for Controller<D, M> {
    type Error = Error<D>;

    fn receive_frame<const N: usize>(&mut self) -> Result<Frame<N>, Error<D>> {
        let mut frame = Frame::default();
        loop {
            for b in &mut self.rx_buffer {
                if self.decoder.decode(b, &mut frame)? == State::Complete {
                    return Ok(frame);
                }
            }

            let n = self
                .rx_buffer
                .read_from(&mut self.driver)
                .map_err(|e| Error::Driver(e))?;

            // se read ritorna 0 significa che driver è arrivato ad EoF.
            if n == 0 {
                return Err(Error::Eof);
            }
        }
    }

    fn send_frame<const N: usize>(&mut self, frame: &Frame<N>) -> Result<usize, Error<D>> {
        let n = Encoder::encode(&frame, self.tx_buffer.as_mut_slice())?;
        self.tx_buffer.set_indexes(0, n);
        self.tx_buffer
            .write_to(n, &mut self.driver)
            .map_err(|e| Error::Driver(e))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use embedded_io::{Error, ErrorType, Read, Write};
    use thiserror::Error;

    #[derive(Debug, PartialEq, Eq, PartialOrd, Ord)]
    pub struct TestDriver<const N: usize>(Buffer<N>);

    #[derive(Error, Debug, PartialEq, Eq, PartialOrd, Ord)]
    pub enum TestDriverError {
        #[error("Write error")]
        Write,
    }

    impl Error for TestDriverError {
        fn kind(&self) -> embedded_io::ErrorKind {
            embedded_io::ErrorKind::Other
        }
    }

    impl<const N: usize> ErrorType for TestDriver<N> {
        type Error = TestDriverError;
    }

    impl<const N: usize> Read for TestDriver<N> {
        fn read(&mut self, buf: &mut [u8]) -> Result<usize, Self::Error> {
            let Ok(n) = self.0.read(buf);
            Ok(n)
        }
    }

    impl<const N: usize> Write for TestDriver<N> {
        fn write(&mut self, buf: &[u8]) -> Result<usize, Self::Error> {
            let Ok(n) = self.0.write(buf);
            Ok(n)
        }

        fn flush(&mut self) -> Result<(), Self::Error> {
            Ok(())
        }
    }

    impl<const N: usize> Driver for TestDriver<N> {
        type Endpoint = Option<[u8; N]>;

        fn connect(enpoint: Self::Endpoint) -> Result<Self, Self::Error>
        where
            Self: Sized,
        {
            let buffer = match enpoint {
                None => Buffer::default(),
                Some(buf) => Buffer::with(buf),
            };

            Ok(TestDriver(buffer))
        }
    }

    #[test]
    fn test_drive_reading() {
        let mut driver = TestDriver::connect(Some([0, 1, 2, 3, 4, 5, 6, 7, 8])).unwrap();
        let mut buf: [u8; 4] = [0; 4];
        let n = driver.read(&mut buf).unwrap();
        assert_eq!(n, 4);
        assert_eq!(buf, [0, 1, 2, 3]);
        let n = driver.read(&mut buf).unwrap();
        assert_eq!(n, 4);
        assert_eq!(buf, [4, 5, 6, 7]);
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
            decoder: Decoder::default(),
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
            decoder: Decoder::default(),
        };
        let frame = controller.receive_frame().unwrap();
        assert_eq!(frame, expected_frame);
    }

    #[test]
    fn receive_double_simple_frame() {
        let io_stream = [
            0x32, 0x43, 0xFE, 0xFE, 0xA4, 0xE0, 0x07, 0x00, 0xFD, 0xFE, 0xFE, 0xE0, 0xA4, 0xFB,
            0xFD,
        ];
        let payload: [u8; 4] = [0x07, 0x00, 0x00, 0x00];
        let expected_frame1 = Frame {
            da: 0xA4,
            sa: 0xE0,
            payload,
            payload_len: 2,
        };
        let expected_frame2 = Frame {
            da: 0xE0,
            sa: 0xA4,
            payload: [0xFB, 0x00, 0x00, 0x00],
            payload_len: 1,
        };
        let mut controller: Controller<TestDriver<15>, 2> = Controller {
            driver: TestDriver::connect(Some(io_stream)).unwrap(),
            radio: Address(0xA4),
            controller: Address(0xE0),
            echo_back: false,
            rx_buffer: Buffer::default(),
            tx_buffer: Buffer::default(),
            decoder: Decoder::default(),
        };
        let frame = controller.receive_frame().unwrap();
        assert_eq!(frame, expected_frame1);

        let frame = controller.receive_frame().unwrap();
        assert_eq!(frame, expected_frame2);
    }

    #[test]
    fn receive_simple_frame_long_preamble() {
        let io_stream = [
            0x32, 0x43, 0xFE, 0xFE, 0xFE, 0xFE, 0xFE, 0xFE, 0xFE, 0xFE, 0xFE, 0xFE, 0xFE, 0xFE,
            0xFE, 0xFE, 0xA4, 0xE0, 0x07, 0x00, 0xFD,
        ];
        let payload: [u8; 8] = [0x07, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00];
        let expected_frame = Frame {
            da: 0xA4,
            sa: 0xE0,
            payload,
            payload_len: 2,
        };
        let mut controller: Controller<TestDriver<_>, 2> = Controller {
            driver: TestDriver::connect(Some(io_stream)).unwrap(),
            radio: Address(0xA4),
            controller: Address(0xE0),
            echo_back: false,
            rx_buffer: Buffer::default(),
            tx_buffer: Buffer::default(),
            decoder: Decoder::default(),
        };
        let frame = controller.receive_frame().unwrap();
        assert_eq!(frame, expected_frame);
    }

    #[test]
    fn receive_simple_frame_eof() {
        let io_stream = [0x32, 0x43, 0xFE, 0xFE, 0xA4, 0xE0, 0x07, 0x00];
        let mut controller: Controller<TestDriver<_>, 2> = Controller {
            driver: TestDriver::connect(Some(io_stream)).unwrap(),
            radio: Address(0xA4),
            controller: Address(0xE0),
            echo_back: false,
            rx_buffer: Buffer::default(),
            tx_buffer: Buffer::default(),
            decoder: Decoder::default(),
        };
        assert_eq!(controller.receive_frame::<8>(), Err(super::Error::Eof));
    }

    #[test]
    fn send_ng_message() {
        let frame = Frame {
            da: 0xE0,
            sa: 0xA4,
            payload: [0xFA, 0x00, 0x00, 0x00],
            payload_len: 1,
        };
        let expected_stream: [u8; 8] = [0xFE, 0xFE, 0xE0, 0xA4, 0xFA, 0xFD, 0x00, 0x00];

        let mut controller: Controller<TestDriver<8>, 8> = Controller {
            driver: TestDriver::connect(None).unwrap(),
            radio: Address(0xA4),
            controller: Address(0xE0),
            echo_back: false,
            rx_buffer: Buffer::default(),
            tx_buffer: Buffer::default(),
            decoder: Decoder::default(),
        };
        let n = controller.send_frame(&frame).unwrap();
        assert_eq!(n, 6);
        assert_eq!(controller.driver.0.as_slice(), expected_stream);
    }

    #[test]
    fn send_simple_frame() {
        // IC-705 CI-V Ref p17 Operating frequency
        // 145.225 MHz
        let payload: [u8; 16] = [0x05, 0, 0, 5, 0, 2, 2, 4, 5, 0, 1, 0, 0, 0, 0, 0];
        let frame = Frame {
            da: 0xA4,
            sa: 0xE0,
            payload,
            payload_len: 11,
        };
        let expected_stream: [u8; 16] = [
            0xFE, 0xFE, 0xA4, 0xE0, 0x05, 0x00, 0x00, 0x05, 0x00, 0x02, 0x02, 0x04, 0x05, 0x00,
            0x01, 0xFD,
        ];
        let mut controller: Controller<TestDriver<16>, 16> = Controller {
            driver: TestDriver::connect(None).unwrap(),
            radio: Address(0xA4),
            controller: Address(0xE0),
            echo_back: false,
            rx_buffer: Buffer::default(),
            tx_buffer: Buffer::default(),
            decoder: Decoder::default(),
        };
        let n = controller.send_frame(&frame).unwrap();
        assert_eq!(n, 16);
        assert_eq!(controller.driver.0.as_slice(), expected_stream);
    }
}
