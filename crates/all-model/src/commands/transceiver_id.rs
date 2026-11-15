use ci_v::{command::Command, radio::Model, types::TransceiverId};

pub fn read_transceiver_id() -> ReadTransceiverID {
    ReadTransceiverID(Vec::new())
}

#[derive(Debug)]
pub struct ReadTransceiverID(Vec<u8>);

impl<M: Model> Command<M> for ReadTransceiverID {
    const Cn: u8 = 0x19;

    const Sc: &'static [u8] = &[0x0];

    const N: usize = 4;

    type Response = TransceiverId;

    fn encode(&mut self) -> &[u8] {
        self.0 = Vec::with_capacity(<Self as Command<M>>::N);
        self.0.push(<Self as Command<M>>::Cn);
        self.0.extend_from_slice(<Self as Command<M>>::Sc);
        self.0.as_slice()
    }
}
