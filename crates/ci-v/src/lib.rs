/// Bytes è una sequenza di byte che forniscono una rappresenzazione di basso
/// livello di comandi e messaggi del protocollo CI-V.
pub struct Bytes(pub(crate) Vec<u8>);

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

/// RadioAddress è il byte identificativo di una radio Icom. Ogni radio ha un
/// valore preimpostato che può essere cambiato mediante i menù della radio.
pub struct RadioAddress(pub(crate) u8);

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
    radio_address: RadioAddress,
    kind: MessageKind,
}

/// MessageBuilder permette di costruire un messaggio con un comando da inviare
/// alla radio con indirizzo *radio_address*.
pub struct MessageBuilder {
    radio_address: RadioAddress,
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
    pub fn to_radio(radio_address: RadioAddress) -> MessageBuilder {
        MessageBuilder { radio_address }
    }

    pub fn from_radio(bytes: Bytes) -> Self {
        todo!()
    }
}
