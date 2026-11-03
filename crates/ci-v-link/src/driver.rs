use crate::frame::Frame;

/// Driver rappresenta lo strato fisico della connessione tra controller e
/// radio e permette di inviare e ricevere trame `Frame`.
pub trait Driver {
    type Error;
    type Frame: Frame;

    /// receive_frame riceve un singolo Frame dal link che il driver ha stabilito
    /// con la radio.
    fn receive_frame(&mut self) -> Result<Self::Frame, Self::Error>;

    /// send_frame invia sul link il Frame fornito come argomento.
    fn send_frame(&mut self, frame: &Self::Frame) -> Result<usize, Self::Error>;
}
