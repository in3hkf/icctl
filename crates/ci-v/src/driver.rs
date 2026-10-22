use thiserror::Error;

use crate::{Address, types::TransceiverId};

/// Packet rappresenta il messaggio scambiato a livello di driver tra la radio
/// e il controller. Tutti i comandi delle radio Icom implementeranno il trait
/// into Packet<N>.
pub struct Packet<const N: usize> {
    pub(crate) id: u8,
    pub(crate) load: [u8; N],
}

// Il trait Radio permette di identificare univocamente una specifica radio
// Icom, fornendo `name`, `default_address` e `transceiver_id`. Inoltre permette
// di ottenere l'indirizzo del controller attraverso il metodo `controller_address`
// che è già implementato per restituire l'indirizzo di default 0xE0.
pub trait Radio {
    fn name(&self) -> &'static str;
    fn default_address(&self) -> Address;
    fn transceiver_id(&self) -> TransceiverId;
    fn controller_address(&self) -> Address {
        Address(0xE0)
    }
}

#[derive(Debug, Error, PartialEq, Eq, PartialOrd, Ord)]
pub enum Error {}

/// Driver è l'interfaccia tra la radio Icom e il controller permettendo di
/// inviare e ricevere pacchetti. Il metodo `connect` crea la connessione e
/// restituisce l'istanza specifica di `Driver`. Il tipo `Endpoint` permette
/// di specificare, per ogni particolare tipologia di `Driver`, le informazioni
/// necessarie ad perfezionare il collegamento. Ad esempio, se il `Driver` usa
/// la porta seriale USB per collegarsi alla radio, `Endpoint` fornirà il path
/// specifico della porta USB come `/dev/cu.usbmodem11203` o `/dev/ttyUSB0`.
pub trait Driver {
    type Endpoint;

    fn connect<R: Radio>(radio: R, enpoint: Self::Endpoint) -> Result<Self, Error>
    where
        Self: Sized;

    fn send<const N: usize, P: Into<Packet<N>>>(&self, packet: P) -> Result<(), Error>;

    fn receive<const N: usize>(&self) -> Result<Packet<N>, Error>;
}
