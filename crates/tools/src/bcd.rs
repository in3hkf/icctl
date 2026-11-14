use thiserror::Error;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Error)]
pub enum BcdError {
    /// Un nibble contiene un valore > 9.
    #[error("nibble BCD non valido (byte 0x{byte:02X}) alla posizione {index}")]
    InvalidNibble { byte: u8, index: usize },
    /// Il valore decodificato non entra nel tipo di destinazione.
    #[error("valore BCD fuori range per il tipo")]
    Overflow,
}

/// Conversione da/verso BCD packed, big-endian, a lunghezza minima.
pub trait Bcd: Sized {
    fn to_bcd(self) -> Vec<u8>;
    fn from_bcd(bytes: &[u8]) -> Result<Self, BcdError>;
}

macro_rules! impl_bcd {
    ($($t:ty),* $(,)?) => {$(
        impl Bcd for $t {
            fn to_bcd(mut self) -> Vec<u8> {
                let mut out = Vec::new();
                loop {
                    let lo = (self % 10) as u8;
                    self /= 10;
                    let hi = (self % 10) as u8;
                    self /= 10;
                    out.push((hi << 4) | lo);
                    if self == 0 {
                        break;
                    }
                }
                out.reverse();
                out
            }

            fn from_bcd(bytes: &[u8]) -> Result<Self, BcdError> {
                let mut value: $t = 0;
                for (index, &byte) in bytes.iter().enumerate() {
                    let hi = byte >> 4;
                    let lo = byte & 0x0F;
                    if hi > 9 || lo > 9 {
                        return Err(BcdError::InvalidNibble { byte, index });
                    }
                    value = value
                        .checked_mul(100)
                        .and_then(|v| v.checked_add((hi * 10 + lo) as $t))
                        .ok_or(BcdError::Overflow)?;
                }
                Ok(value)
            }
        }
    )*};
}

impl_bcd!(u8, u16, u32, u64);

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn encode_u8() {
        assert_eq!(0u8.to_bcd(), vec![0x00]);
        assert_eq!(9u8.to_bcd(), vec![0x09]);
        assert_eq!(42u8.to_bcd(), vec![0x42]);
        assert_eq!(99u8.to_bcd(), vec![0x99]);
        assert_eq!(100u8.to_bcd(), vec![0x01, 0x00]);
        assert_eq!(255u8.to_bcd(), vec![0x02, 0x55]);
    }

    #[test]
    fn encode_wider_types() {
        assert_eq!(123u16.to_bcd(), vec![0x01, 0x23]);
        assert_eq!(12345u16.to_bcd(), vec![0x01, 0x23, 0x45]);
        assert_eq!(1_234_567u32.to_bcd(), vec![0x01, 0x23, 0x45, 0x67]);
        assert_eq!(
            12_345_678_901_234u64.to_bcd(),
            vec![0x12, 0x34, 0x56, 0x78, 0x90, 0x12, 0x34]
        );
    }

    #[test]
    fn decode_valid() {
        assert_eq!(u8::from_bcd(&[0x42]), Ok(42));
        assert_eq!(u8::from_bcd(&[0x02, 0x55]), Ok(255));
        assert_eq!(u16::from_bcd(&[0x01, 0x23]), Ok(123));
        assert_eq!(u32::from_bcd(&[0x01, 0x23, 0x45, 0x67]), Ok(1_234_567));
    }

    #[test]
    fn decode_leading_zeros_and_empty() {
        assert_eq!(u8::from_bcd(&[0x00, 0x12]), Ok(12));
        assert_eq!(u16::from_bcd(&[0x00, 0x00, 0x07]), Ok(7));
        assert_eq!(u32::from_bcd(&[]), Ok(0));
    }

    #[test]
    fn decode_invalid_nibble() {
        assert_eq!(
            u8::from_bcd(&[0x1A]),
            Err(BcdError::InvalidNibble {
                byte: 0x1A,
                index: 0
            })
        );
        assert_eq!(
            u16::from_bcd(&[0x12, 0xA3]),
            Err(BcdError::InvalidNibble {
                byte: 0xA3,
                index: 1
            })
        );
        assert_eq!(
            u32::from_bcd(&[0x01, 0x23, 0xFF]),
            Err(BcdError::InvalidNibble {
                byte: 0xFF,
                index: 2
            })
        );
    }

    #[test]
    fn decode_overflow() {
        assert_eq!(u8::from_bcd(&[0x02, 0x56]), Err(BcdError::Overflow));
        assert_eq!(u16::from_bcd(&[0x06, 0x55, 0x36]), Err(BcdError::Overflow));
        assert_eq!(
            u32::from_bcd(&[0x42, 0x94, 0x96, 0x72, 0x96]),
            Err(BcdError::Overflow)
        );
        assert_eq!(
            u64::from_bcd(&[
                0x01, 0x84, 0x46, 0x74, 0x40, 0x73, 0x70, 0x95, 0x51, 0x61, 0x6
            ]),
            Err(BcdError::Overflow)
        );
    }

    #[test]
    fn roundtrip_limits() {
        assert_eq!(u8::from_bcd(&u8::MAX.to_bcd()), Ok(u8::MAX));
        assert_eq!(u16::from_bcd(&u16::MAX.to_bcd()), Ok(u16::MAX));
        assert_eq!(u32::from_bcd(&u32::MAX.to_bcd()), Ok(u32::MAX));
        assert_eq!(u64::from_bcd(&u64::MAX.to_bcd()), Ok(u64::MAX));
        assert_eq!(u64::from_bcd(&0u64.to_bcd()), Ok(0));
    }

    #[test]
    fn roundtrip_exhaustive_u8_u16() {
        for v in 0..=u8::MAX {
            assert_eq!(u8::from_bcd(&v.to_bcd()), Ok(v));
        }
        for v in 0..=u16::MAX {
            assert_eq!(u16::from_bcd(&v.to_bcd()), Ok(v));
        }
    }

    #[test]
    fn error_messages() {
        let e = BcdError::InvalidNibble {
            byte: 0xA3,
            index: 1,
        };
        assert_eq!(
            e.to_string(),
            "nibble BCD non valido (byte 0xA3) alla posizione 1"
        );
        assert_eq!(
            BcdError::Overflow.to_string(),
            "valore BCD fuori range per il tipo"
        );
    }
}
