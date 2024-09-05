#![no_std]

pub extern crate alloc;

use alloc::string::String;
use serde::{Deserialize, Serialize};

pub use derive;
pub use postcard::{from_bytes as deserialize, to_allocvec as serialize};

#[cfg(feature = "embedded")]
pub mod embedded;

#[derive(Serialize, Deserialize)]
pub struct Event {
    pub name: String,
}
