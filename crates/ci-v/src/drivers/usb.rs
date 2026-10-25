use std::println;

use alloc::{boxed::Box, format, string::String};
use serialport::SerialPort;
use thiserror::Error;

use crate::driver::Driver;

#[derive(Error, Debug)]
pub enum Error {
    #[error(transparent)]
    SerialPort(#[from] serialport::Error),
    #[error("Reading from USB reports: {0}")]
    Read(String),
    #[error("Writing to USB reports: {0}")]
    Write(String),
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
    type Endpoint = String;

    type Error = Error;

    fn connect(endpoint: Self::Endpoint) -> Result<Self, Self::Error>
    where
        Self: Sized,
    {
        let port = serialport::new(&endpoint, 115200).open()?;
        println!("Port: {:?}", port.name());
        Ok(UsbPort { endpoint, port })
    }

    fn read(&mut self, buf: &mut [u8]) -> Result<usize, Self::Error> {
        self.port.read(buf).map_err(|e| Error::Read(format!("{e}")))
    }

    fn write(&mut self, buf: &[u8]) -> Result<usize, Self::Error> {
        self.port
            .write(buf)
            .map_err(|e| Error::Write(format!("{e}")))
    }
}
