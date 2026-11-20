use std::convert::Infallible;

use ci_v::command::{self, Bytes, Command, Response};
use thiserror::Error;

use crate::{
    Ic705,
    commands::formats::{Channel, ChannelError, ChannelGroup, ChannelGroupError, ChannelType},
};

pub fn read_memory() -> ReadContent<Empty> {
    ReadContent(Empty)
}

#[derive(Debug)]
pub struct ReadContent<Mode>(Mode);

pub struct Empty;

pub struct WithGroup(ChannelGroup);

impl ReadContent<Empty> {
    pub fn group(self, group: u8) -> Result<ReadContent<WithGroup>, ChannelGroupError> {
        let cg = ChannelGroup::group(group)?;
        Ok(ReadContent(WithGroup(cg)))
    }

    pub fn call_group(self) -> ReadContent<WithGroup> {
        let ccg = ChannelGroup::call();
        ReadContent(WithGroup(ccg))
    }
}

#[derive(Debug)]
pub struct WithGroupAndChannel {
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
                    unreachable!("function is_call_channel return false, but Channel Type is Call");
                };
                Err(ReadContentError::NoCallChannelCallGroup(ch, group.0))
            }
            (false, true) => {
                let ChannelType::Call(ch) = ch.0 else {
                    unreachable!(
                        "function is_call_channel return true, but Channel Type is NoCall"
                    );
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

    type Response = MemoryContent;

    fn encode(&mut self) -> Bytes {
        command::Encoder::with_cn(Self::Cn)
            .sc(Self::Sc)
            .format(&self.0.group.to_bcd())
            .format(&self.0.channel.to_bcd())
            .encode()
    }
}

#[derive(Debug)]
pub struct MemoryContent;

impl Response for MemoryContent {
    type Error = Infallible;

    fn response(payload: &[u8]) -> Result<Self, Self::Error> {
        println!("ReadContent Response payload: {payload:X?}");
        println!("payload length: {}", payload.len());
        Ok(MemoryContent)
    }
}
