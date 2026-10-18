use thiserror::Error;

use crate::link::Frame;

const PREAMBLE: u8 = 0xFE;
const TERMINATOR: u8 = 0xFD;

#[derive(Error, Clone, Copy, Debug, PartialEq, Eq)]
pub enum Error {
    #[error("Target Space insufficient (expected {expected:?}, found {found:?})")]
    TargetSpace { expected: usize, found: usize },
}

pub struct Codec;

impl Codec {
    pub fn encode(&self, frame: &Frame, target: &mut [u8]) -> Result<usize, Error> {
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
        let codec = Codec {};
        let n = codec.encode(&f, &mut buf).unwrap();
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
        let codec = Codec {};
        assert_eq!(
            codec.encode(&f, &mut buf),
            Err(Error::TargetSpace {
                expected: 8,
                found: 4
            })
        );
    }
}
