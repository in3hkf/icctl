use serialport::Result;
use serialport::available_ports;

// Sul mio mac ottengo per il IC-705:
//
//   Port: /dev/cu.usbmodem11201
//   	 UsbPort(
//       UsbPortInfo {
//           vid: 0x0c26,
//           pid: 0x0036,
//           serial_number: Some(
//               "IC-705 13002785",
//           ),
//           manufacturer: Some(
//               "Icom Inc.",
//           ),
//           product: Some(
//               "IC-705",
//           ),
//       },
//   )
//   Port: /dev/tty.usbmodem11201
//   	 UsbPort(
//       UsbPortInfo {
//           vid: 0x0c26,
//           pid: 0x0036,
//           serial_number: Some(
//               "IC-705 13002785",
//           ),
//           manufacturer: Some(
//               "Icom Inc.",
//           ),
//           product: Some(
//               "IC-705",
//           ),
//       },
//   )
//   Port: /dev/cu.usbmodem11203
//   	 UsbPort(
//       UsbPortInfo {
//           vid: 0x0c26,
//           pid: 0x0036,
//           serial_number: Some(
//               "IC-705 13002785",
//           ),
//           manufacturer: Some(
//               "Icom Inc.",
//           ),
//           product: Some(
//               "IC-705",
//           ),
//       },
//   )
//   Port: /dev/tty.usbmodem11203
//   	 UsbPort(
//       UsbPortInfo {
//           vid: 0x0c26,
//           pid: 0x0036,
//           serial_number: Some(
//               "IC-705 13002785",
//           ),
//           manufacturer: Some(
//               "Icom Inc.",
//           ),
//           product: Some(
//               "IC-705",
//           ),
//       },
//   )
//
// For controlling the Icom IC-705 via IC-V software on macOS, you should typically
// use one of the /dev/tty.usbmodem... ports rather than the /dev/cu.usbmodem... ports.
//
// Recommended choice: /dev/tty.usbmodem11201 or /dev/tty.usbmodem11203
//
// The distinction between these port types on macOS:
//
// /dev/cu.* (callout ports) — Designed for outgoing connections
// /dev/tty.* (teletype ports) — Standard for serial data communication and incoming connections
// For radio control applications like IC-V, the tty ports are the conventional choice for CAT serial communication.
//
// Between 11201 and 11203, they likely represent different virtual COM interfaces exposed by
// the same physical device (the IC-705 creates multiple serial endpoints).
// Try /dev/tty.usbmodem11201 first as it's typically the primary interface.
// If that doesn't work, switch to 11203.
//
// Note that you may also need to ensure:
//
// The IC-705 is in CAT control mode (menu settings)
// Proper baud rate configuration in IC-V (typically 19200 or 38400 bps for IC-705)
// That USB drivers for Icom devices are properly installed
// Did the IC-V software give you any error messages when trying to connect?
// That could help determine which specific port interface the IC-705 is using for control data.

pub fn main() -> Result<()> {
    available_ports()?.iter().for_each(|port| {
        println!("Port: {}", port.port_name);
        println!("\t {:#?}", port.port_type);
    });

    Ok(())
}
