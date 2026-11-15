use thiserror::Error;
use tools::bcd::Bcd;

/// Memory group number
/// 0000 ~ 0099: Memory channel group
///        0100: Call channel group
#[derive(Debug)]
pub struct ChannelGroup(pub(crate) u8);

#[derive(Error, Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
#[error("Valid Channel Groups are between 0 and 99. Group {0} is not valid.")]
pub struct ChannelGroupError(u8);

impl ChannelGroup {
    /// group ritorna il `ChannelGroup` specificato nel parametro `group`se tale
    /// valore rientra nel range ammesso: 0000 - 0099. Altrimenti la funzione
    /// ritorna l'errore `ChannelGroupError`.
    pub fn group(group: u8) -> Result<Self, ChannelGroupError> {
        match group {
            n @ _ if n <= 100 => Ok(Self(n)),
            m @ _ => Err(ChannelGroupError(m)),
        }
    }

    /// call ritorna il gruppo di canali associato alle frequenze `Call`.
    /// Tale gruppo ha necessariamente valore 100.
    pub fn call() -> Self {
        Self(100)
    }

    #[inline]
    pub(crate) fn is_call_group(&self) -> bool {
        self.0 == 100
    }

    pub fn to_bcd(&self) -> Vec<u8> {
        if self.is_call_group() {
            return self.0.to_bcd();
        }

        let mut gr = self.0.to_bcd();
        gr.insert(0, 0x00);

        gr
    }
}

#[doc(hidden)]
#[derive(Debug)]
pub(crate) enum ChannelType {
    Call(u8),
    NoCall(u8),
}

/// Memory channel numbers
/// • When Memory channel group is selected,
///     0000 ~ 0099: 00 ~ 99
/// • When Call channel group is selected,
///     0000, 0001: 144 C1, C2
///     0002, 0003: 430 C1, C2
#[derive(Debug)]
pub struct Channel(pub(crate) ChannelType);

#[derive(Error, Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
#[error("Valid Channels are between 0 and 99. Channel {0} is not valid.")]
pub struct ChannelError(u8);

#[derive(Error, Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
#[error("Valid Call Channels are between 0 and 3. Call Channel {0} is not valid.")]
pub struct CallChannelError(u8);

impl Channel {
    /// channel ritorna il `Channel` specificato nel parametro `channel` se tale
    /// valore rientra nel range ammesso: 0000 - 0099. Altrimenti la funzione
    /// ritorna l'errore `ChannelError`.
    pub fn channel(channel: u8) -> Result<Self, ChannelError> {
        match channel {
            n @ _ if n <= 100 => Ok(Self(ChannelType::NoCall(n))),
            m @ _ => Err(ChannelError(m)),
        }
    }

    /// call ritorna il canale associato alle frequenze `Call`.
    /// Tale canale ha necessariamente uno dei valori:
    ///
    /// | Banda | C1 | C2 |
    /// |-------|----|----|
    /// | 144   | 0  | 1  |
    /// | 433   | 2  | 3  |
    ///
    pub fn call(channel: u8) -> Result<Self, CallChannelError> {
        match channel {
            n @ _ if n <= 3 => Ok(Self(ChannelType::Call(n))),
            m @ _ => Err(CallChannelError(m)),
        }
    }

    #[inline]
    pub(crate) fn is_call_channel(&self) -> bool {
        match self.0 {
            ChannelType::Call(_) => true,
            ChannelType::NoCall(_) => false,
        }
    }

    pub fn to_bcd(&self) -> Vec<u8> {
        match self.0 {
            ChannelType::Call(c) => {
                let mut ch = c.to_bcd();
                if ch.len() == 1 {
                    ch.insert(0, 0x00);
                }
                ch
            }
            ChannelType::NoCall(c) => {
                let mut ch = c.to_bcd();
                if ch.len() == 1 {
                    ch.insert(0, 0x00);
                }
                ch
            }
        }
    }
}
