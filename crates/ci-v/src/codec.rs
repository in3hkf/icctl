use thiserror::Error;

use crate::link::Frame;

const PREAMBLE: u8 = 0xFE;
const TERMINATOR: u8 = 0xFD;

#[derive(Error, Clone, Copy, Debug, PartialEq, Eq)]
pub enum Error {
    #[error("Target Space insufficient (expected {expected:?}, found {found:?})")]
    TargetSpace { expected: usize, found: usize },
}

pub struct Encoder;

impl Encoder {
    pub fn encode<const N: usize>(frame: &Frame<N>, target: &mut [u8]) -> Result<usize, Error> {
        // Il messaggio CI-V è formato da:
        // [FE][FE][Dst][Src][Cn][Sc][Data Area][FD]
        // Frame rappresenta la parte da Dst a Data Area, quindi la dimensione
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
        // preamble2cmd è composto dai due byte di preamble (FE), da [Dst], [Src] e
        // [Cn]: quindi dal preamble al cmd (Cn).
        let (preamble2cmd, rest) = msg.split_first_chunk_mut::<5>().ok_or(target_space_error)?;
        // load è composto da [Sc][Data Area]
        let (load, terminator) = rest.split_last_chunk_mut::<1>().ok_or(target_space_error)?;

        *preamble2cmd = [PREAMBLE, PREAMBLE, frame.dst, frame.src, frame.cmd];
        load.copy_from_slice(&frame.load[..frame.load_len]);
        *terminator = [TERMINATOR];

        Ok(total_len)
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum State {
    #[default]
    Idle,
    Preamble,
    Dst,
    Src,
    Cmd,
    Load,
    Complete,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Error)]
pub enum DecodeError {
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

#[derive(Debug, Default)]
pub struct Decoder {
    state: State,
}

impl Decoder {
    pub const fn new() -> Self {
        Self { state: State::Idle }
    }

    /// `feed` alimenta il Decoder con il successivo byte ricevuto dallo stream
    /// di ingresso. Tale byte viene interpretato in funzione dello stato
    /// interno della macchina a stati del decoder e viene, eventualmente,
    /// aggiornato lo stato. Tale stato viene restituito dalla funzione stessa.
    /// Nel caso il decoder abbia a disposizione un Frame completo il suo stato
    /// viene impostato a Complete.
    pub fn decode<const N: usize>(
        &mut self,
        b: u8,
        frame: &mut Frame<N>,
    ) -> Result<State, DecodeError> {
        use State::*;
        // dopo un frame completo, Decoder reimposta lo stato a Idle.
        if self.state == Complete {
            self.state = Idle;
        }
        match (self.state, b) {
            (Idle, PREAMBLE) => self.state = Preamble,
            (Idle, _) => {}
            (Preamble, PREAMBLE) => self.state = Dst,
            (Preamble, _) => self.state = Idle,
            (Dst, PREAMBLE) => {} // preambolo esteso (più di due FE)
            (Dst | Src | Cmd, TERMINATOR) => {
                self.state = Idle;
                return Err(DecodeError::TooShort);
            }
            (Src | Cmd | Load, PREAMBLE) => {
                // l'FE appena visto è già il primo del preambolo successivo
                self.state = Preamble;
                return Err(DecodeError::Truncated);
            }
            (Dst, _) => {
                frame.dst = b;
                self.state = Src;
            }
            (Src, _) => {
                frame.src = b;
                self.state = Cmd;
            }
            (Cmd, _) => {
                frame.cmd = b;
                frame.load_len = 0;
                self.state = Load;
            }
            (Load, TERMINATOR) => {
                self.state = Complete;
                return Ok(Complete);
            }
            (Load, _) => match frame.load.get_mut(frame.load_len) {
                Some(slot) => {
                    *slot = b;
                    frame.load_len += 1;
                }
                None => {
                    self.state = Idle;
                    return Err(DecodeError::Overflow);
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
            dst: 0xA4,
            src: 0xE0,
            cmd: 0x03,
            load: [0; 16],
            load_len: 0,
        };
        let mut buf = [0u8; 16];
        let n = Encoder::encode(&f, &mut buf).unwrap();
        assert_eq!(&buf[..n], &[0xFE, 0xFE, 0xA4, 0xE0, 0x03, 0xFD]);
    }

    #[test]
    fn rejects_small_buffer() {
        let f = Frame {
            dst: 0xA4,
            src: 0xE0,
            cmd: 0x03,
            load: [0x01, 0x02],
            load_len: 2,
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
            (frame.dst, frame.src, frame.cmd, frame.load),
            (0xA4, 0xE0, 0x03, [0x01, 0x02])
        );
    }
}
