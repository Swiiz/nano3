use serde::{Deserialize, Serialize};

pub use derive;
pub use postcard::from_bytes as deserialize;

#[derive(Serialize, Deserialize)]
pub struct Event {
    pub name: String,
}
