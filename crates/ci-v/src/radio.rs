use crate::{Address, Link, types::TransceiverId};

/// Model permette di identificare in maniera univoca un modello specifico di
/// radio Icom. Una radio Icom ha un valore univoco, ed unico specifico del
/// modello stesso, chiamato transceiver ID. Tale valore è restituito dal
/// comando CI-V 19 00, e coincide con il valore di default del indirizzo della
/// radio stessa. In questo trait aggiungiamo anche un altro parametro che
/// permette di avere il nome del modello, come ad esempio IC-705.
pub trait Model {
    const TRANSCEIVER_ID: u8;
    const NAME: &'static str;

    #[inline]
    fn model() -> &'static str {
        Self::NAME
    }

    #[inline]
    fn is_this_model(transceiver_id: TransceiverId) -> bool {
        transceiver_id.0 == Self::TRANSCEIVER_ID
    }
}

/// Radio
pub struct Radio<M: Model, L: Link> {
    model: M,
    link: L,
    radio_address: Option<Address>,
    controller_address: Address,
}
