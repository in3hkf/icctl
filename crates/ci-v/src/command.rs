use core::ops::Deref;

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
    fn encode(&mut self) -> Bytes;
}

/// Sequenza di bytes che codifica un comando CI-V:
/// Tale sequenza contiene Cn ed, eventualmente, Sc e Data nei comandi che fanno
/// uso di tali valori.
#[derive(Debug, Clone)]
pub struct Bytes(Vec<u8>);

/// Si crea una versione codificata di un comando con [`Encoder::with_cn`], che
/// richiede il comando `Cn`, e poi si concatenano nell'ordine corretto i campi
/// opzionali `Sc` e i `format` che compongono `Data`. L'ordine è imposto dal
/// sistema dei tipi (typestate): un ordine errato non compila.
/// Si chiude con `done()`.
///
/// ```ignore
/// let bytes = Encoder::with_cn(0x1A)
///     .sc(&[0x05])
///     .format(&group.to_bcd())
///     .format(&channel.to_bcd())
///     .encode();
/// ```
#[derive(Debug, Clone)]
pub struct Encoder;

impl Encoder {
    /// Avvia la codifica con il codice comando `Cn`.
    ///
    /// È l'unico punto d'ingresso: non può esistere un `Octets` senza `Cn`.
    pub fn with_cn(cn: u8) -> WithCn {
        let mut v = Vec::with_capacity(16);
        v.push(cn);
        WithCn(v)
    }
}

/// Stato della codifica: è stato inserito solo `Cn`.
///
/// Da qui si può proseguire con `sc`, con `format`, oppure chiudere con `done`.
#[derive(Debug)]
pub struct WithCn(Vec<u8>);

/// Stato della codifica: sono stati inseriti `Cn` e `Sc`.
///
/// Da qui si può proseguire con `format`, oppure chiudere con `done`.
#[derive(Debug)]
pub struct WithSc(Vec<u8>);

/// Stato della codifica: sono stati inseriti `Cn`, l'eventuale `Sc` e almeno
/// un `format` di `Data`.
///
/// Da qui si possono aggiungere altri `format`, oppure chiudere con `done`.
#[derive(Debug)]
pub struct WithData(Vec<u8>);

impl WithCn {
    /// Aggiunge il subcommand `Sc` (opzionale).
    ///
    /// Può essere chiamato al più una volta, subito dopo `Cn`.
    pub fn sc(mut self, sc: &[u8]) -> WithSc {
        self.0.extend_from_slice(sc);
        WithSc(self.0)
    }

    /// Aggiunge il primo `format` del campo `Data`, senza `Sc`.
    pub fn format(mut self, f: &[u8]) -> WithData {
        self.0.extend_from_slice(f);
        WithData(self.0)
    }

    /// Termina la codifica di un comando composto dal solo `Cn`.
    pub fn encode(self) -> Bytes {
        Bytes(self.0)
    }
}

impl WithSc {
    /// Aggiunge il primo `format` del campo `Data`.
    pub fn format(mut self, f: &[u8]) -> WithData {
        self.0.extend_from_slice(f);
        WithData(self.0)
    }

    /// Termina la codifica di un comando composto da `Cn + Sc`.
    pub fn encode(self) -> Bytes {
        Bytes(self.0)
    }
}

impl WithData {
    /// Aggiunge un ulteriore `format` al campo `Data`.
    ///
    /// Può essere chiamato quante volte serve: i byte vengono concatenati
    /// nell'ordine delle chiamate.
    pub fn format(mut self, f: &[u8]) -> Self {
        self.0.extend_from_slice(f);
        self
    }

    /// Termina la codifica di un comando completo `Cn + Sc? + Data`.
    pub fn encode(self) -> Bytes {
        Bytes(self.0)
    }
}

impl Deref for Bytes {
    type Target = [u8];

    /// Accesso in sola lettura ai byte codificati.
    fn deref(&self) -> &[u8] {
        &self.0
    }
}
