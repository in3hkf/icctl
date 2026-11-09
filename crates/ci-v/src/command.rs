use std::error::Error;

use thiserror::Error;

use crate::radio::Model;

/// Response rappresenta la risposta ottenuta dai comandi del protocollo CI-V.
/// Esistono comandi che rispondono con un messaggio che è una versione, anche,
/// estesa del comando inviato come, ad esempio, i comandi di Read di uno degli
/// stati o configurazioni della radio. Oppure, la risposta può essere un Ack,
/// che può assumero solo due forme OK o NG (No Good).
pub trait Response {
    /// Response definisce il tipo di risposta che viene ottenuta dalla richiesta
    /// di questo comando.
    type Response;

    /// Error che può accadere nella decodifica della risposta.
    type Error: Error;

    /// response esegue la decodifica da una sequenza di byte e, se non si verificano
    /// errori, restituisce un'istanza di Response.
    fn response(payload: &[u8]) -> Result<Self::Response, Self::Error>;
}

/// Command è uno dei possibili comandi del protocollo CI-V delle radio Icom.
pub trait Command: Sized + Response {
    /// Model permette di definire per quale modello di radio tale comando è
    /// implementato.
    type Model: Model;

    /// Command number identifica un comando o un gruppo di comandi, ognuno dei
    /// quali vieni distinto usando un Sub command number `Sc`.
    #[allow(non_upper_case_globals)]
    const Cn: u8;

    /// Sub command number.
    #[allow(non_upper_case_globals)]
    const Sc: Option<u32>;

    /// N è la dimensione massima necessaria per serializzare tale comando in
    /// un buffer.
    const N: usize;

    /// encode esegue la codifica del comando come sequenza di byte utile al invio,
    /// come richiesta alla radio, al driver.
    fn encode(&self) -> &[u8];
}

/// Ack è la risposta che molti comandi CI-V restituiscono
pub enum Ack {
    Ok,
    Ng,
}

#[derive(Error, Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct AckError<'a>(&'a [u8]);

impl std::fmt::Display for AckError<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Ack payload: {:X?}", self.0)
    }
}

impl Ack {
    pub(crate) fn response(payload: &[u8]) -> Result<Ack, AckError<'_>> {
        match payload[2] {
            0xFA => Ok(Ack::Ng),
            0xFB => Ok(Ack::Ok),
            _ => Err(AckError(payload)),
        }
    }
}
