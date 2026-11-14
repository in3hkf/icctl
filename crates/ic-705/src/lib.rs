use ci_v::radio::Model;

pub mod commands;

/// Radio IC-705
pub struct Ic705;

// Modello Icom IC-705 HF/VHF/UHF All Mode Transceiver
impl Model for Ic705 {
    const Transceiver_Id: u8 = 0xA4;

    const Name: &'static str = "IC-705";
}
