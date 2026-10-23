#![no_std]

#[cfg(feature = "alloc")]
extern crate alloc;

#[cfg(feature = "std")]
extern crate std;

mod buffer;
// mod codec;
pub mod driver;
mod link;
mod types;

pub use types::Address;
