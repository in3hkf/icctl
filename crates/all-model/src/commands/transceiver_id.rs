use ci_v::{
    command::{self, Bytes, Command},
    radio::Model,
    types::TransceiverId,
};

pub fn read_transceiver_id() -> ReadTransceiverID {
    ReadTransceiverID
}

#[derive(Debug)]
pub struct ReadTransceiverID;

impl<M: Model> Command<M> for ReadTransceiverID {
    const Cn: u8 = 0x19;

    const Sc: &'static [u8] = &[0x0];

    const N: usize = 2;

    type Response = TransceiverId;

    fn encode(&mut self) -> Bytes {
        command::Encoder::with_cn(<Self as Command<M>>::Cn)
            .sc(<Self as Command<M>>::Sc)
            .encode()
    }
}
