/// Command è uno dei possibili comandi del protocollo CI-V delle radio Icom.
pub trait Command {
    /// Command number identifica un comando o un gruppo di comandi, ognuno dei
    /// quali vieni distinto usando un Sub command number `Sc`.
    #[allow(non_upper_case_globals)]
    const Cn: u8;
    /// Sub command number.
    #[allow(non_upper_case_globals)]
    const Sc: u32;

    /// N è la dimensione massima necessaria per serializzare tale comando in
    /// un buffer.
    const N: usize;
}
