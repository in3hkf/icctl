use std::convert::Infallible;

use ci_v::command::{Command, Response};
use thiserror::Error;

use crate::Ic705;

/// Memory group number
/// 0000 ~ 0099: Memory channel group
///        0100: Call channel group
#[derive(Debug)]
pub struct ChannelGroup(u8);

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
    fn is_call_group(&self) -> bool {
        self.0 == 100
    }
}

#[doc(hidden)]
#[derive(Debug)]
enum ChannelType {
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
pub struct Channel(ChannelType);

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
    fn is_call_channel(&self) -> bool {
        match self.0 {
            ChannelType::Call(_) => true,
            ChannelType::NoCall(_) => false,
        }
    }
}

#[derive(Debug)]
pub struct ReadContent<Mode>(Mode);

struct Empty;

struct WithGroup(ChannelGroup);

impl ReadContent<Empty> {
    pub fn group(group: u8) -> Result<ReadContent<WithGroup>, ChannelGroupError> {
        let cg = ChannelGroup::group(group)?;
        Ok(ReadContent(WithGroup(cg)))
    }

    pub fn call_group() -> ReadContent<WithGroup> {
        let ccg = ChannelGroup::call();
        ReadContent(WithGroup(ccg))
    }
}

#[derive(Debug)]
struct WithGroupAndChannel {
    group: ChannelGroup,
    channel: Channel,
}

#[derive(Error, Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum ReadContentError {
    #[error("Channel error: {0}")]
    Channel(#[from] ChannelError),

    #[error("Using Call Channel {0} in a no Channel Group {1}")]
    CallChannelNoCallGroup(u8, u8),

    #[error("Using No Call Channel {0} in a Call Channel Group {1}")]
    NoCallChannelCallGroup(u8, u8),
}

impl ReadContent<WithGroup> {
    pub fn channel(
        self,
        channel: u8,
    ) -> Result<ReadContent<WithGroupAndChannel>, ReadContentError> {
        let ch = Channel::channel(channel)?;
        let WithGroup(group) = self.0;
        match (group.is_call_group(), ch.is_call_channel()) {
            (true, true) | (false, false) => {
                Ok(ReadContent(WithGroupAndChannel { group, channel: ch }))
            }
            (true, false) => {
                let ChannelType::NoCall(ch) = ch.0 else {
                    panic!("function is_call_channel return false, but Channel Type is Call");
                };
                Err(ReadContentError::NoCallChannelCallGroup(ch, group.0))
            }
            (false, true) => {
                let ChannelType::Call(ch) = ch.0 else {
                    panic!("function is_call_channel return true, but Channel Type is NoCall");
                };
                Err(ReadContentError::CallChannelNoCallGroup(ch, group.0))
            }
        }
    }
}

impl Command<Ic705> for ReadContent<WithGroupAndChannel> {
    const Cn: u8 = 0x1A;

    const Sc: &'static [u8] = &[0x00];

    const N: usize = 6;

    fn encode(&mut self) -> &[u8] {
        todo!()
    }
}

impl Response for ReadContent<WithGroupAndChannel> {
    type Response = ();

    type Error = Infallible;

    fn response(payload: &[u8]) -> Result<Self::Response, Self::Error> {
        todo!()
    }
}
