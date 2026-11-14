use std::{
    io::{BufRead, BufReader, Read, Write},
    println,
};

use ci_v_link::{
    driver::Driver,
    frame::{CodecError, Decoder, Encoder, Frame as FrameTrait, State},
};
use serialport::SerialPort;
use std::{boxed::Box, string::String, vec::Vec};
use thiserror::Error;

const EOM_CODE: u8 = 0xFD;

#[derive(Error, Debug)]
pub enum Error {
    #[error(transparent)]
    SerialPort(#[from] serialport::Error),
    #[error("I/O error from USB reports: {0}")]
    Io(#[from] std::io::Error),
    #[error("unexpected end of stream")]
    Eof,
    #[error("Codec error: {0}")]
    Codec(#[from] CodecError),
}

/// Frame è la PDU del livello Data Link la cui rappresentazione nel protocollo
/// CI-V è la seguente:
///
/// ```text
/// +----------+---------+-----+
/// | PREAMBLE | PAYLOAD | EOM |
/// +----------+---------+-----+
/// 0          2        N+2  N+1
/// ```
///
/// | Campo    | Dimensione | Descrizione                |
/// |----------|------------|----------------------------|
/// | PREAMBLE | 2 Byte     | Preambolo: due byte FE     |
/// | PAYLOAD  | N Byte     | Payload: CI-V Cn+Sc+Data   |
/// | EOM      | 1 Byte     | End Of Message: un byte FD |
#[derive(Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct Frame {
    pub payload: Vec<u8>,
    pub payload_len: usize,
}

impl ci_v_link::frame::Frame for Frame {
    fn with_payload<P: AsRef<[u8]>>(payload: P) -> Self {
        let payload_ref = payload.as_ref();
        let payload_len = payload_ref.len();

        Frame {
            payload: payload_ref.to_vec(),
            payload_len,
        }
    }

    fn payload(&self) -> &[u8] {
        &self.payload
    }
}

#[derive(Debug)]
pub struct UsbPort {
    endpoint: String,
    decoder: Decoder,
    reader: BufReader<Box<dyn SerialPort>>,
}

impl UsbPort {
    pub fn name(&self) -> Option<String> {
        self.reader.get_ref().name()
    }
}

impl Driver for UsbPort {
    type Error = Error;
    type Frame = Frame;

    fn receive_frame(&mut self) -> Result<Self::Frame, Self::Error> {
        let mut payload = vec![0u8; Self::BUFSIZE];
        let mut rx_buffer = vec![0u8; Self::BUFSIZE];

        loop {
            match self.reader.read_until(EOM_CODE, &mut rx_buffer) {
                Ok(n) => {
                    // se read_until ritorna 0 significa che driver è arrivato ad EoF.
                    if n == 0 {
                        return Err(Error::Eof);
                    }

                    for b in rx_buffer.iter() {
                        if let State::Complete(n) =
                            self.decoder.decode(*b, payload.as_mut_slice())?
                        {
                            return Ok(Frame::with_payload(&payload[..n]));
                        }
                    }
                }
                Err(e) => match e.kind() {
                    std::io::ErrorKind::TimedOut => {
                        eprintln!("Timeout");
                        continue;
                    }
                    err @ _ => {
                        eprintln!("I/O Error: {err}");
                        return Err(Error::from(e));
                    }
                },
            }
        }
    }

    fn send_frame(&mut self, frame: &Self::Frame) -> Result<usize, Self::Error> {
        let mut tx_buffer = [0; 1024];
        let n = Encoder::encode(frame, tx_buffer.as_mut_slice())?;
        // println!("Sending Frame payload: {:X?}", &tx_buffer[0..n]);
        let m = self.reader.get_mut().write(&tx_buffer[0..n])?;
        Ok(m)
    }
}

impl UsbPort {
    pub fn connect(endpoint: String) -> Result<Self, Error>
    where
        Self: Sized,
    {
        let port = serialport::new(&endpoint, 115200).open()?;
        println!("Port: {:?}", port.name());
        Ok(UsbPort {
            endpoint,
            decoder: Decoder::default(),
            reader: BufReader::new(port),
        })
    }
}
