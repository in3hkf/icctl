/// Driver rappresenta lo strato fisico della connessione tra controller e
/// radio e permette di inviare e ricevere pacchetti.
/// Il metodo `connect` crea la connessione e restituisce l'istanza specifica
/// di `Driver`. Il tipo `Endpoint` permette di specificare, per ogni
/// particolare tipologia di `Driver`, le informazioni necessarie a perfezionare
/// il collegamento. Ad esempio, se il `Driver` usa la porta seriale USB per
/// collegarsi alla radio, `Endpoint` fornirà il path specifico della porta USB
/// come `/dev/cu.usbmodem11203` o `/dev/ttyUSB0`.
pub trait Driver {
    type Endpoint;
    type Error;

    fn connect(enpoint: Self::Endpoint) -> Result<Self, Self::Error>
    where
        Self: Sized;

    fn read(&mut self, buf: &mut [u8]) -> Result<usize, Self::Error>;

    fn write(&mut self, buf: &[u8]) -> Result<usize, Self::Error>;
}
