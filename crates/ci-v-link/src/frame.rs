use thiserror::Error;

const PREAMBLE_CODE: u8 = 0xFE;
const EOM_CODE: u8 = 0xFD;

/// Frame è la PDU del livello Data Link la cui rappresentazione nel protocollo
/// CI-V è la seguente:
///
/// ```text
/// +----------+---------+-----+
/// | PREAMBLE | PAYLOAD | EOM |
/// +----------+---------+-----+
/// 0          2        N+2  N+1
/// ```
///
/// | Campo    | Dimensione | Descrizione                |
/// |----------|------------|----------------------------|
/// | PREAMBLE | 2 Byte     | Preambolo: due byte FE     |
/// | PAYLOAD  | N Byte     | Payload: CI-V Cn+Sc+Data   |
/// | EOM      | 1 Byte     | End Of Message: un byte FD |
///
/// Il trait Frame è l'interfaccia tra il Driver e lo strato superiore.
pub trait Frame {
    /// permette di costruire un'istanza di Frame con il `payload` passato come
    /// parametro.
    fn with_payload<P: AsRef<[u8]>>(payload: P) -> Self;

    /// restituisce il `payload` di tale Frame come slice.
    fn payload(&self) -> &[u8];
}

#[derive(Error, Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum CodecError {
    #[error("Target Space insufficient (expected {expected:?}, found {found:?})")]
    TargetSpace { expected: usize, found: usize },
    #[error("frame too short")]
    // FD prima di cmd
    TooShort,
    #[error("truncated frame")]
    // FE dentro il frame: il precedente è interrotto
    Truncated,
    #[error("load exceeds decoder buffer")]
    // load più grande del buffer
    Overflow,
}

pub struct Encoder;

impl Encoder {
    pub fn encode<F: Frame>(frame: &F, target: &mut [u8]) -> Result<usize, CodecError> {
        // Il messaggio CI-V è formato da:
        // [FE][FE][Da][Sa][Cn][Sc][Data Area][FD]
        // Frame rappresenta la parte da Da a Data Area, quindi la dimensione
        // totale del messaggio è data dalla dimensione di Frame più i tre byte
        // del preambolo, FE FE, e il byte terminatore FD.
        let total_len = frame.payload().len() + 3;

        // Errore ritornato da `encode` in caso mancanza di spazio in `target`.
        // Deve essere Copy perchè ci sono tre invocazioni a `ok_or`.
        let target_space_error = CodecError::TargetSpace {
            expected: total_len,
            found: target.len(),
        };

        // Verifico che target abbia lunghezza almeno di `total_len` byte.
        let msg = target.get_mut(..total_len).ok_or(target_space_error)?;
        // preamble è composto dai due byte di preamble (FE)
        let (preamble, rest) = msg.split_first_chunk_mut::<2>().ok_or(target_space_error)?;
        // payload è composto da [Da][Sa][Cn][Sc][Data Area]
        let (payload, terminator) = rest.split_last_chunk_mut::<1>().ok_or(target_space_error)?;

        *preamble = [PREAMBLE_CODE, PREAMBLE_CODE];
        payload.copy_from_slice(&frame.payload());
        *terminator = [EOM_CODE];

        Ok(total_len)
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum State {
    #[default]
    Idle,
    Preamble,
    Payload(usize),
    Complete,
}

#[derive(Debug, Default)]
pub struct Decoder {
    state: State,
}

impl Decoder {
    pub const fn new() -> Self {
        Self { state: State::Idle }
    }

    /// `decode` alimenta il Decoder con il successivo byte ricevuto dallo stream
    /// di ingresso. Tale byte viene interpretato in funzione dello stato
    /// interno della macchina a stati del decoder e viene, eventualmente,
    /// aggiornato lo stato. Tale stato viene restituito dalla funzione stessa.
    /// Nel caso il decoder abbia a disposizione un Frame completo il suo stato
    /// viene impostato a Complete e nella slice frame_payload è contenuto il
    /// payload del Frame.
    pub fn decode(&mut self, b: u8, frame_payload: &mut [u8]) -> Result<State, CodecError> {
        use State::*;

        // dopo un frame completo, Decoder reimposta lo stato a Idle.
        if self.state == Complete {
            self.state = Idle;
        }
        match (self.state, b) {
            (Idle, PREAMBLE_CODE) => self.state = Preamble,
            (Idle, _) => {}
            (Preamble, PREAMBLE_CODE) => self.state = Payload(0),
            (Preamble, _) => self.state = Idle,
            (Payload(n), PREAMBLE_CODE) if n == 0 => {} // preambolo esteso (più di due FE)
            (Payload(_), PREAMBLE_CODE) => {
                // l'FE appena visto è già il primo del preambolo successivo
                self.state = Preamble;
                return Err(CodecError::Truncated);
            }
            (Payload(n), EOM_CODE) if n <= 2 => {
                self.state = Idle;
                return Err(CodecError::TooShort);
            }
            (Payload(_), EOM_CODE) => {
                self.state = Complete;
                return Ok(Complete);
            }
            (Payload(n), _) => match frame_payload.get_mut(n) {
                Some(slot) => {
                    *slot = b;
                    self.state = Payload(n + 1);
                }
                None => {
                    self.state = Idle;
                    return Err(CodecError::Overflow);
                }
            },
            (Complete, _) => {} // già gestito sopra
        }
        Ok(self.state)
    }
}
