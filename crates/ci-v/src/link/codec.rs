use thiserror::Error;

use crate::link::Frame;

const PREAMBLE_CODE: u8 = 0xFE;
const EOM_CODE: u8 = 0xFD;

#[derive(Error, Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum Error {
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
    pub fn encode<const N: usize>(frame: &Frame<N>, target: &mut [u8]) -> Result<usize, Error> {
        // Il messaggio CI-V è formato da:
        // [FE][FE][Da][Sa][Cn][Sc][Data Area][FD]
        // Frame rappresenta la parte da Da a Data Area, quindi la dimensione
        // totale del messaggio è data dalla dimensione di Frame più i tre byte
        // del preambolo, FE FE, e il byte terminatore FD.
        let total_len = frame.len() + 3;

        // Errore ritornato da `encode` in caso mancanza di spazio in `target`.
        // Deve essere Copy perchè ci sono tre invocazioni a `ok_or`.
        let target_space_error = Error::TargetSpace {
            expected: total_len,
            found: target.len(),
        };

        // Verifico che target abbia lunghezza almeno di `total_len` byte.
        let msg = target.get_mut(..total_len).ok_or(target_space_error)?;
        // preambleadds è composto dai due byte di preamble (FE), da [Da] e [Sa]:
        // quindi da preamble più indirizzi.
        let (preambleadds, rest) = msg.split_first_chunk_mut::<4>().ok_or(target_space_error)?;
        // load è composto da [Cn][Sc][Data Area]
        let (payload, terminator) = rest.split_last_chunk_mut::<1>().ok_or(target_space_error)?;

        *preambleadds = [PREAMBLE_CODE, PREAMBLE_CODE, frame.da, frame.sa];
        payload.copy_from_slice(&frame.payload[..frame.payload_len]);
        *terminator = [EOM_CODE];

        Ok(total_len)
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum State {
    #[default]
    Idle,
    Preamble,
    Da,
    Sa,
    Payoad,
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
    /// viene impostato a Complete.
    pub fn decode<const N: usize>(&mut self, b: u8, frame: &mut Frame<N>) -> Result<State, Error> {
        use State::*;
        // dopo un frame completo, Decoder reimposta lo stato a Idle.
        if self.state == Complete {
            self.state = Idle;
        }
        match (self.state, b) {
            (Idle, PREAMBLE_CODE) => self.state = Preamble,
            (Idle, _) => {}
            (Preamble, PREAMBLE_CODE) => self.state = Da,
            (Preamble, _) => self.state = Idle,
            (Da, PREAMBLE_CODE) => {} // preambolo esteso (più di due FE)
            (Da | Sa, EOM_CODE) => {
                self.state = Idle;
                return Err(Error::TooShort);
            }
            (Sa | Payoad, PREAMBLE_CODE) => {
                // l'FE appena visto è già il primo del preambolo successivo
                self.state = Preamble;
                return Err(Error::Truncated);
            }
            (Da, _) => {
                frame.da = b;
                self.state = Sa;
            }
            (Sa, _) => {
                frame.sa = b;
                frame.payload_len = 0;
                self.state = Payoad;
            }
            (Payoad, EOM_CODE) => {
                self.state = Complete;
                return Ok(Complete);
            }
            (Payoad, _) => match frame.payload.get_mut(frame.payload_len) {
                Some(slot) => {
                    *slot = b;
                    frame.payload_len += 1;
                }
                None => {
                    self.state = Idle;
                    return Err(Error::Overflow);
                }
            },
            (Complete, _) => {} // già gestito sopra
        }
        Ok(self.state)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn encodes_read_frequency_request() {
        let f = Frame {
            da: 0xA4,
            sa: 0xE0,
            payload: [0x03, 0x00, 0x00, 0x00],
            payload_len: 1,
        };
        let mut buf = [0u8; 16];
        let n = Encoder::encode(&f, &mut buf).unwrap();
        assert_eq!(&buf[..n], &[0xFE, 0xFE, 0xA4, 0xE0, 0x03, 0xFD]);
    }

    #[test]
    fn rejects_small_buffer() {
        let f = Frame {
            da: 0xA4,
            sa: 0xE0,
            payload: [0x03, 0x01, 0x02],
            payload_len: 3,
        };
        let mut buf = [0u8; 4];
        assert_eq!(
            Encoder::encode(&f, &mut buf),
            Err(Error::TargetSpace {
                expected: 8,
                found: 4
            })
        );
    }

    #[test]
    fn byte_by_byte_with_noise() {
        let mut d = Decoder::new();
        let bytes = [
            0x00, 0xFE, 0x01, 0xFE, 0xFE, 0xA4, 0xE0, 0x03, 0x01, 0x02, 0xFD,
        ];
        let mut complete_at = None;
        let mut frame = Frame::default();
        for (i, &b) in bytes.iter().enumerate() {
            if d.decode(b, &mut frame).unwrap() == State::Complete {
                complete_at = Some(i);
            }
        }
        assert_eq!(complete_at, Some(bytes.len() - 1));
        assert_eq!(
            (frame.da, frame.sa, frame.payload),
            (0xA4, 0xE0, [0x03, 0x01, 0x02])
        );
    }
}
