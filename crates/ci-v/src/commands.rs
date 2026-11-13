use thiserror::Error;

use crate::{
    command::{Command, Response},
    radio::Model,
    types::TransceiverId,
};

#[derive(Error, Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum Error {
    #[error("Response with wrong length: {0}")]
    Length(usize),
}

pub fn read_transceiver_id() -> ReadTransceiverID<Req> {
    ReadTransceiverID(Req(Vec::new()))
}

#[derive(Debug)]
pub struct ReadTransceiverID<T>(T);

#[derive(Debug)]
pub struct Req(Vec<u8>);

impl<M: Model> Command<M> for ReadTransceiverID<Req> {
    const Cn: u8 = 0x19;

    const Sc: &'static [u8] = &[0x0];

    const N: usize = 4;

    fn encode(&mut self) -> &[u8] {
        self.0.0 = Vec::with_capacity(<Self as Command<M>>::N);
        self.0.0.push(<Self as Command<M>>::Cn);
        self.0.0.extend_from_slice(<Self as Command<M>>::Sc);
        self.0.0.as_slice()
    }
}

#[derive(Debug)]
pub struct Res(TransceiverId);

impl Response for ReadTransceiverID<Req> {
    type Response = ReadTransceiverID<Res>;

    type Error = Error;

    fn response(payload: &[u8]) -> Result<Self::Response, Self::Error> {
        let length = payload.len();
        // println!("Response payload: {payload:X?}");
        // Response payload: [E0, A4, 19, 0, A4]
        if length != 5 {
            return Err(Error::Length(length));
        }
        let tid = payload[length - 1];
        Ok(ReadTransceiverID(Res(TransceiverId(tid))))
    }
}

impl core::fmt::Display for ReadTransceiverID<Res> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Read Transceiver ID response: {:X}", self.0.0)
    }
}
