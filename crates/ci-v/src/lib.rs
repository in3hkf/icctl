mod codec;
mod link;
mod types;

use serialport::SerialPort;

use crate::types::{Address, Bytes, TransceiverId};

pub trait Model {
    fn name(&self) -> &'static str;
    fn transceiver_id(&self) -> &'static TransceiverId;
    fn default_address(&self) -> &'static Address;
}

/// Link rappresenta gli estremi del collegamento tra la radio Icom e il suo
/// controller. Il collegamento avviene tramite seriale.
pub(crate) struct Link {
    // path è il path assoluto del dispositivo seriale esposto dalla radio.
    path: String,
    // radio_address è l'indirizzo CI-V al quale risponde la radio collegata da
    // tale link.
    radio_address: Address,
    // ctrl_address è l'indirizzo del controller che gestisce tale link.
    ctrl_address: Address,
    // connection è l'effettiva connessione seriale attraverso la quale radio e
    // controller dialogano.
    connection: Option<Box<dyn SerialPort>>,
}

/// Radio è una radio Icom di modello Model.
/// Scopo di Radio è quello di attivare un collegamento seriale con il modello
/// di radio Icom Model alla quale invierà comandi.
pub struct Radio<M: Model> {
    // model è il modello specifico di radio Icom che si desidera interfacciare.
    model: M,
    // link è il collegamento, e i suoi estremi, che permette il dialogo tra il
    // controller e la radio reale.
    link: Link,
}

/// Command rappresenta un comando CI-V con il seguente formato:
///
/// [Cn][Sc][Data area]
///
/// Dove:
///  - Cn è il Command number ed è sempre presente.
///  - Sc è il Sub command number ed è presente per alcuni Cn.
///  - Data area contiene gli argomenti del comando. E' presente in alcuni
///    comandi.
pub trait Command {
    type M: Model;

    fn payload(&self) -> Bytes;
}

// MessageKind permette di specificare una delle quattro possibili tipologie di
// messaggio dle protocollo CI-V.
pub(crate) enum MessageKind {
    // Messaggio dal Controller alla Radio
    ToRadio(Bytes),
    // Messaggio dalla Radio al Controller
    FromRadio(Bytes),
    // Messaggio di OK dalla Radio al Controller
    Ok(),
    // Messaggio di NG dalla Radio al Controller
    NG(),
}

/// Message è un messaggio di protocollo CI-V scambiato tra i due nodi
/// terminali: una radio Icom e il controller. Esistono, fondamentalmente,
/// quattro tipologie di messaggio.
///  - Dal controller alla radio:
///      [FE][FE][Radio-Address][E0][Command][FD]
///  - Dalla radio al controller:
///      [FE][FE][E0][Radio-Address][Command][FD]
///  - Messaggio di OK della radio al controller:
///      [FE][FE][E0][Radio-Address][FB][FD]
///  - Messaggio NG dalla radio al controller:
///      [FE][FE][E0][Radio-Address][FA][FD]
pub struct Message {
    radio_address: Address,
    kind: MessageKind,
}

/// MessageBuilder permette di costruire un messaggio con un comando da inviare
/// alla radio con indirizzo *radio_address*.
pub struct MessageBuilder {
    radio_address: Address,
}

impl MessageBuilder {
    /// command fornisce il comando che permette di definire nella sua interezza
    /// il Message da inviare alla radio.
    pub fn command<C: Command>(self, command: C) -> Message {
        let MessageBuilder { radio_address } = self;

        Message {
            radio_address,
            kind: MessageKind::ToRadio(command.payload()),
        }
    }
}

impl Message {
    /// to_radio permette di costruire un messaggio che il controller invierà
    /// alla radio con indirizzo *radio_address*. Tale funzione restituisce un
    /// builder di Message.
    pub fn to_radio(radio_address: Address) -> MessageBuilder {
        MessageBuilder { radio_address }
    }

    pub fn from_radio(bytes: Bytes) -> Self {
        todo!()
    }
}
