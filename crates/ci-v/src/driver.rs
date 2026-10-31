use embedded_io::{Read, Write};

/// Driver rappresenta lo strato fisico della connessione tra controller e
/// radio e permette di inviare e ricevere pacchetti.
/// Il metodo `connect` crea la connessione e restituisce l'istanza specifica
/// di `Driver`. Il tipo `Endpoint` permette di specificare, per ogni
/// particolare tipologia di `Driver`, le informazioni necessarie a perfezionare
/// il collegamento. Ad esempio, se il `Driver` usa la porta seriale USB per
/// collegarsi alla radio, `Endpoint` fornirà il path specifico della porta USB
/// come `/dev/cu.usbmodem11203` o `/dev/ttyUSB0`.
pub trait Driver: Read + Write {
    type Endpoint;

    fn connect(endpoint: Self::Endpoint) -> Result<Self, Self::Error>
    where
        Self: Sized;
}
