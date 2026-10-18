/// Address è il byte identificativo di una radio Icom o del Controller.
/// Ogni radio ha un valore preimpostato che può essere cambiato mediante i menù
/// della radio.
pub struct Address(pub(crate) u8);

/// TransceiverId è il valore identificativo dello specifico modello di radio Icom.
/// Tale valore, non modificabile dai menù della radio, è quello restituito dal
/// comando CI-V 19 00.
pub struct TransceiverId(pub(crate) u8);
