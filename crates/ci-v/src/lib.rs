#![no_std]

#[cfg(feature = "alloc")]
extern crate alloc;

#[cfg(feature = "std")]
extern crate std;

mod buffer;
mod command;
pub mod driver;
pub mod drivers;
mod link;
mod radio;
mod types;

pub use link::*; // Temporaneo per simple.rs.
pub use types::Address;
