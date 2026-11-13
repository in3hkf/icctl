use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};

use ci_v::Ic705;
use ci_v::Radio;
use ci_v::command::{Command, Response};
use ci_v::commands::read_transceiver_id;
use ci_v::drivers::usb::UsbPort;
use ci_v_link::driver::Driver;

pub fn main() -> core::result::Result<(), String> {
    let running = Arc::new(AtomicBool::new(true));
    let r = running.clone();
    ctrlc::set_handler(move || r.store(false, Ordering::SeqCst)).map_err(|e| format!("{e}"))?;

    let ep = "/dev/cu.usbmodem11201".into();

    let mut driver = UsbPort::connect(ep).map_err(|e| format!("{e}"))?;
    let mut ic705 = Radio::with_model(Ic705).driver(driver);

    let res = ic705
        .exec(read_transceiver_id())
        .map_err(|e| format!("{e}"))?;

    println!("Response: {res}");

    Ok(())
}
