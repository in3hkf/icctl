use core::fmt::Display;

use thiserror::Error;

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
pub struct TransceiverId(pub(crate) u8);
