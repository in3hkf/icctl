use core::fmt::Display;
use std::fmt::UpperHex;

use thiserror::Error;

use crate::command::Response;

/// Address è il byte identificativo di una radio Icom o del Controller.
/// Ogni radio ha un valore preimpostato che può essere cambiato mediante i menù
/// della radio.
pub struct Address(pub(crate) u8);

#[derive(Debug, Error)]
pub struct InvalidAddress;

impl Display for InvalidAddress {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(
            f,
            "Invalid CI-V Address: 0xFE or 0xFD are not valid addresses"
        )
    }
}

impl Address {
    /// 0xFE (preambolo) e 0xFD (terminatore) non sono indirizzi validi.
    pub const fn new(addr: u8) -> Result<Self, InvalidAddress> {
        match addr {
            0xFD | 0xFE => Err(InvalidAddress),
            _ => Ok(Self(addr)),
        }
    }
    pub const fn get(&self) -> u8 {
        self.0
    }
}

/// TransceiverId è il valore identificativo dello specifico modello di radio Icom.
/// Tale valore, non modificabile dai menù della radio, è quello restituito dal
/// comando CI-V 19 00.
#[derive(Debug)]
pub struct TransceiverId(pub u8);

impl core::fmt::Display for TransceiverId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Transceiver ID: {:X}", self.0)
    }
}

#[derive(Error, Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum Error {
    #[error("Response with wrong length: {0}")]
    Length(usize),
}

impl Response for TransceiverId {
    type Error = Error;

    fn response(payload: &[u8]) -> Result<Self, Self::Error> {
        let length = payload.len();
        // println!("Response payload: {payload:X?}");
        // Response payload: [19, 0, A4]
        if length != 3 {
            return Err(Error::Length(length));
        }
        let tid = payload[length - 1];
        Ok(TransceiverId(tid))
    }
}

impl TransceiverId {
    pub const fn get(&self) -> u8 {
        self.0
    }
}

impl UpperHex for TransceiverId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:X}", self.0)
    }
}
