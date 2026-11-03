use std::println;

use alloc::{boxed::Box, format, string::String, vec::Vec};
use ci_v_link::driver::Driver;
use embedded_io::{ErrorType, Read, Write};
use serialport::SerialPort;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum Error {
    #[error(transparent)]
    SerialPort(#[from] serialport::Error),
    #[error("Reading from USB reports: {0}")]
    Read(String),
    #[error("Writing to USB reports: {0}")]
    Write(String),
}

impl embedded_io::Error for Error {
    fn kind(&self) -> embedded_io::ErrorKind {
        embedded_io::ErrorKind::Other
    }
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

impl ci_v_link::driver::Frame for Frame {
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
        todo!()
    }

    fn send_frame(&mut self, frame: &Self::Frame) -> Result<usize, Self::Error> {
        todo!()
    }
}

impl UsbPort {
    pub fn connect(endpoint: String) -> Result<Self, Error>
    where
        Self: Sized,
    {
        let port = serialport::new(&endpoint, 115200).open()?;
        println!("Port: {:?}", port.name());
        Ok(UsbPort { endpoint, port })
    }
}

impl ErrorType for UsbPort {
    type Error = Error;
}

impl Read for UsbPort {
    fn read(&mut self, buf: &mut [u8]) -> Result<usize, Self::Error> {
        self.port.read(buf).map_err(|e| Error::Read(format!("{e}")))
    }
}

impl Write for UsbPort {
    fn write(&mut self, buf: &[u8]) -> Result<usize, Self::Error> {
        self.port
            .write(buf)
            .map_err(|e| Error::Write(format!("{e}")))
    }

    fn flush(&mut self) -> Result<(), Self::Error> {
        self.port.flush().map_err(|e| Error::Write(format!("{e}")))
    }
}
