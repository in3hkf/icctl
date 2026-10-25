use ci_v::{Controller, Frame, driver::Driver, drivers::usb::UsbPort};
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};

pub fn main() -> core::result::Result<(), String> {
    let running = Arc::new(AtomicBool::new(true));
    let r = running.clone();
    ctrlc::set_handler(move || r.store(false, Ordering::SeqCst)).map_err(|e| format!("{e}"))?;

    let ep = "/dev/cu.usbmodem1201".into();

    let mut controller: Controller<UsbPort> =
        Controller::with(UsbPort::connect(ep).map_err(|e| format!("{e}"))?);

    // IC-705 CI-V Ref p17 Operating frequency
    // 145.225 MHz
    let payload: [u8; 8] = [0x05, 0, 0x50, 0x22, 0x45, 0x01, 0, 0];
    // let payload: [u8; 8] = [0x05, 0, 0x80, 0x34, 0x69, 0x01, 0, 0];
    let frame = Frame {
        da: 0xA4,
        sa: 0xE0,
        payload,
        payload_len: 6,
    };

    // IC-705 CI-V Ref p17 Reading Operating frequency
    // let payload: [u8; 4] = [0x03, 0, 0, 0];
    // let frame = Frame {
    //     da: 0xA4,
    //     sa: 0xE0,
    //     payload,
    //     payload_len: 1,
    // };

    // IC-705 CI-V Ref p17 Operating Mode
    // let payload: [u8; 4] = [0x06, 3, 3, 0];
    // let frame = Frame {
    //     da: 0xA4,
    //     sa: 0xE0,
    //     payload,
    //     payload_len: 3,
    // };

    println!("Sending frame: \n{frame}");
    println!("Sending frame: \n{frame}");

    match controller.send_frame(&frame) {
        Ok(n) => println!("Send frame Ok: {n}"),
        Err(e) => eprintln!("Send frame reports: {e}"),
    }

    while running.load(Ordering::SeqCst) {
        match controller.receive_frame::<64>() {
            Ok(frame) => println!("Receive {frame}"),
            Err(e) => eprintln!("Receiving frame reports: {e}"),
        }
        use std::thread::sleep;
        use std::time::Duration;

        sleep(Duration::from_millis(500));
    }

    println!("Bye!");
    Ok(())
}
