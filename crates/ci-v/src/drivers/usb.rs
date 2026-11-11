use std::{
    io::{Read, Write},
    println,
};

use ci_v_link::{
    driver::Driver,
    frame::{CodecError, Decoder, Encoder, Frame as FrameTrait, State},
};
use serialport::SerialPort;
use std::{boxed::Box, string::String, vec::Vec};
use thiserror::Error;

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
    port: Box<dyn SerialPort>,
    decoder: Decoder,
    rx_buffer: Vec<u8>,
    tx_buffer: Vec<u8>,
}

impl UsbPort {
    pub fn name(&self) -> Option<String> {
        self.port.name()
    }
}

impl Driver for UsbPort {
    type Error = Error;
    type Frame = Frame;

    fn receive_frame(&mut self) -> Result<Self::Frame, Self::Error> {
        let mut payload = Vec::with_capacity(1024); // TODO! >> Cambiare 1024 con const

        loop {
            let mut m: usize = 0;
            for b in self.rx_buffer.iter() {
                m += 1;
                if self.decoder.decode(*b, payload.as_mut_slice())? == State::Complete {
                    self.rx_buffer.drain(..m);
                    return Ok(Frame::with_payload(payload));
                }
            }

            self.rx_buffer.drain(..m);
            let mut buf = [0; 1024]; // TODO! >> Cambiare 1024 con const
            let n = self.port.read(&mut buf)?;
            self.rx_buffer.extend_from_slice(&buf[0..n]);

            // se read ritorna 0 significa che driver è arrivato ad EoF.
            if n == 0 {
                return Err(Error::Eof);
            }
        }
    }

    fn send_frame(&mut self, frame: &Self::Frame) -> Result<usize, Self::Error> {
        let n = Encoder::encode(frame, self.tx_buffer.as_mut_slice())?;
        let m = self.port.write(&self.tx_buffer[0..n])?;
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
            port,
            decoder: Decoder::default(),
            rx_buffer: Vec::new(),
            tx_buffer: Vec::new(),
        })
    }
}
