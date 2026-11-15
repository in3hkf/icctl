use crate::radio::Model;

pub const NG: u8 = 0xFA;
pub const OK: u8 = 0xFB;
pub const EVENT_FREQ: u8 = 0x00;
pub const EVENT_MODE: u8 = 0x01;

/// Response rappresenta la risposta ottenuta dai comandi del protocollo CI-V.
/// Esistono comandi che rispondono con un messaggio che è una versione, anche,
/// estesa del comando inviato come, ad esempio, i comandi di Read di uno degli
/// stati o configurazioni della radio. Oppure, la risposta può essere un Ack,
/// che può assumero solo due forme OK o NG (No Good).
pub trait Response: Sized + std::fmt::Debug {
    /// Error che può accadere nella decodifica della risposta.
    type Error: std::error::Error;

    /// response esegue la decodifica da una sequenza di byte e, se non si verificano
    /// errori, restituisce un'istanza di Response.
    fn response(payload: &[u8]) -> Result<Self, Self::Error>;
}

/// Command è uno dei possibili comandi del protocollo CI-V delle radio Icom.
/// Model permette di definire per quale modello di radio tale comando è
/// implementato.
pub trait Command<M: Model>: Sized + std::fmt::Debug {
    /// Command number identifica un comando o un gruppo di comandi, ognuno dei
    /// quali vieni distinto usando un Sub command number `Sc`.
    #[allow(non_upper_case_globals)]
    const Cn: u8;

    /// Sub command number.
    #[allow(non_upper_case_globals)]
    const Sc: &'static [u8];

    /// N è la dimensione massima necessaria per serializzare tale comando in
    /// un buffer.
    const N: usize;

    /// Response definisce il tipo di risposta che viene ottenuta dalla richiesta
    /// di questo comando.
    type Response: Response;

    /// encode esegue la codifica del comando come sequenza di byte utile al invio,
    /// come richiesta alla radio, al driver.
    fn encode(&mut self) -> &[u8];
}
