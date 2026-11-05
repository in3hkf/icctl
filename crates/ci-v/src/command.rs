use thiserror::Error;

use crate::radio::Model;

#[derive(Error, Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum Error {}

/// Command è uno dei possibili comandi del protocollo CI-V delle radio Icom.
pub trait Command: Sized {
    /// Model permette di definire per quale modello di radio tale comando è
    /// implementato.
    type Model: Model;

    /// Response definisce il tipo di risposta viene ottenuta dalla richiesta
    /// di questo comando.
    type Response;

    /// Command number identifica un comando o un gruppo di comandi, ognuno dei
    /// quali vieni distinto usando un Sub command number `Sc`.
    #[allow(non_upper_case_globals)]
    const Cn: u8;
    /// Sub command number.
    #[allow(non_upper_case_globals)]
    const Sc: u32;

    /// N è la dimensione massima necessaria per serializzare tale comando in
    /// un buffer.
    const N: usize;

    /// encode esegue la codifica del comando come sequenza di byte utile al invio
    /// al driver.
    fn encode(&self) -> &[u8];

    /// decode esegue la decodifica da una sequenza di byte e, se non si verificano
    /// errori, restituisce un'istanza di tale comando.
    fn decode(payload: &[u8]) -> Result<Self, Error>;
}
