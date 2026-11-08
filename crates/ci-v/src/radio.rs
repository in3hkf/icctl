use std::error;

use ci_v_link::{driver::Driver, frame::Frame};
use thiserror::Error;

use crate::{Address, command::Command, types::TransceiverId};

/// Model permette di identificare in maniera univoca un modello specifico di
/// radio Icom. Una radio Icom ha un valore univoco, ed unico specifico del
/// modello stesso, chiamato transceiver ID. Tale valore è restituito dal
/// comando CI-V 19 00, e coincide con il valore di default del indirizzo della
/// radio stessa. In questo trait aggiungiamo anche un altro parametro che
/// permette di avere il nome del modello, come ad esempio IC-705.
pub trait Model {
    #[allow(non_upper_case_globals)]
    const Transceiver_Id: u8;
    #[allow(non_upper_case_globals)]
    const Name: &'static str;

    #[inline]
    fn model() -> &'static str {
        Self::Name
    }

    #[inline]
    fn is_this_model(transceiver_id: TransceiverId) -> bool {
        transceiver_id.0 == Self::Transceiver_Id
    }
}

#[derive(Error, Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum Error<D: Driver, C: Command> {
    #[error("Driver error: $0")]
    Driver(D::Error),

    #[error("Command error: $0")]
    Command(C::Error),
}

/// Radio
pub struct Radio<M, D> {
    model: M,
    driver: D,
    radio_address: Option<Address>,
    controller_address: Option<Address>,
}

struct NoModel;
struct NoDriver;

impl Radio<NoModel, NoDriver> {
    pub fn with_model<M: Model>(model: M) -> Radio<M, NoDriver> {
        Radio {
            model,
            driver: NoDriver,
            radio_address: None,
            controller_address: None,
        }
    }

    pub fn with_driver<D: Driver>(driver: D) -> Radio<NoModel, D> {
        Radio {
            model: NoModel,
            driver,
            radio_address: None,
            controller_address: None,
        }
    }
}

impl<M: Model> Radio<M, NoDriver> {
    pub fn driver<D: Driver>(self, driver: D) -> Radio<M, D> {
        let Self {
            model,
            driver: _,
            radio_address,
            controller_address,
        } = self;

        Radio {
            model,
            driver,
            radio_address,
            controller_address,
        }
    }
}

impl<D: Driver> Radio<NoModel, D> {
    pub fn model<M: Model>(self, model: M) -> Radio<M, D> {
        let Self {
            model: _,
            driver,
            radio_address,
            controller_address,
        } = self;

        Radio {
            model,
            driver,
            radio_address,
            controller_address,
        }
    }
}

impl<M: Model, D: Driver> Radio<M, D> {
    pub fn radio_address(self, address: Address) -> Self {
        Self {
            radio_address: Some(address),
            ..self
        }
    }

    pub fn controller_address(self, address: Address) -> Self {
        Self {
            controller_address: Some(address),
            ..self
        }
    }

    /// exec invia alla Radio il comando
    pub fn exec<C: Command<Model = M>>(
        &mut self,
        cmd: C,
    ) -> Result<C::Response, Error<D, C::Response>> {
        let command = cmd.encode();
        let mut payload: Vec<u8> = Vec::with_capacity(C::N + 5);
        payload.push(self.get_radio_address());
        payload.push(self.get_controller_address());
        payload.extend_from_slice(command);
        let req_frame = <D as Driver>::Frame::with_payload(payload);

        self.driver
            .send_frame(&req_frame)
            .map_err(|e| Error::Driver(e))?;

        let res_frame = self.driver.receive_frame().map_err(|e| Error::Driver(e))?;

        <C as Command>::Response::decode(res_frame.payload()).map_err(|e| Error::Command(e))
    }
}

// Private implementation
impl<M: Model, D: Driver> Radio<M, D> {
    fn get_radio_address(&self) -> u8 {
        match self.radio_address {
            Some(ref addr) => addr.get(),
            None => M::Transceiver_Id,
        }
    }

    fn get_controller_address(&self) -> u8 {
        match self.controller_address {
            Some(ref addr) => addr.get(),
            None => 0xE0,
        }
    }
}
