#![no_std]

pub extern crate alloc;

use alloc::string::String;
use derive::wasm_event;
pub use serde::{de::DeserializeOwned, Deserialize, Serialize};

pub use derive;
pub use postcard::{from_bytes as deserialize, to_allocvec as serialize};

#[cfg(feature = "embedded")]
pub mod embedded;
pub mod event;

#[wasm_event]
pub struct OnStart {
    pub host: String,
}
