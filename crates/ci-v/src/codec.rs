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
    pub fn encode(frame: &Frame, target: &mut [u8]) -> Result<usize, Error> {
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
        load.copy_from_slice(frame.load);
        *terminator = [TERMINATOR];

        Ok(total_len)
    }
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum State {
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

pub struct Decoder<const N: usize> {
    state: State,
    dst: u8,
    src: u8,
    cmd: u8,
    buf: [u8; N],
    len: usize,
}

impl<const N: usize> Decoder<N> {
    pub const fn new() -> Self {
        Self {
            state: State::Idle,
            dst: 0,
            src: 0,
            cmd: 0,
            buf: [0; N],
            len: 0,
        }
    }

    /// `feed` alimenta il Decoder con il successivo byte ricevuto dallo stream
    /// di ingresso. Tale byte viene interpretato in funzione dello stato
    /// interno della macchina a stati del decoder e viene, eventualmente,
    /// aggiornato lo stato. Tale stato viene restituito dalla funzione stessa.
    /// Nel caso il decoder abbia a disposizione un Frame completo il suo stato
    /// viene impostato a Complete.
    pub fn feed(&mut self, b: u8) -> Result<State, DecodeError> {
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
                self.dst = b;
                self.state = Src;
            }
            (Src, _) => {
                self.src = b;
                self.state = Cmd;
            }
            (Cmd, _) => {
                self.cmd = b;
                self.len = 0;
                self.state = Load;
            }
            (Load, TERMINATOR) => {
                self.state = Complete;
                return Ok(Complete);
            }
            (Load, _) => match self.buf.get_mut(self.len) {
                Some(slot) => {
                    *slot = b;
                    self.len += 1;
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

    /// `frame` ritorna Some(Frame) solo subito dopo `feed() == Complete`.
    pub fn frame(&self) -> Option<Frame<'_>> {
        if self.state != State::Complete {
            return None;
        }
        let load = self.buf.get(..self.len)?;
        Some(Frame {
            dst: self.dst,
            src: self.src,
            cmd: self.cmd,
            load,
        })
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
            load: &[],
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
            load: &[0x01, 0x02],
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
        let mut d = Decoder::<16>::new();
        let bytes = [
            0x00, 0xFE, 0x01, 0xFE, 0xFE, 0xA4, 0xE0, 0x03, 0x01, 0x02, 0xFD,
        ];
        let mut complete_at = None;
        for (i, &b) in bytes.iter().enumerate() {
            if d.feed(b).unwrap() == State::Complete {
                complete_at = Some(i);
            }
        }
        assert_eq!(complete_at, Some(bytes.len() - 1));
        let f = d.frame().unwrap();
        assert_eq!(
            (f.dst, f.src, f.cmd, f.load),
            (0xA4, 0xE0, 0x03, &[0x01, 0x02][..])
        );
    }
}
