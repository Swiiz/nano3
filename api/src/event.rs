use alloc::vec::Vec;
use serde::{de::DeserializeOwned, Deserialize, Serialize};

use crate::{deserialize, serialize};

#[derive(Serialize, Deserialize)]
pub struct RawEvent {
    pub unique_id: &'static str,
    data: Vec<u8>,
}

impl RawEvent {
    pub fn from_data<T: AnyEvent>(data: T) -> Self {
        Self {
            unique_id: T::unique_id(),
            data: serialize(&data).unwrap(),
        }
    }

    pub fn is<T: AnyEvent>(&self) -> bool {
        self.unique_id == T::unique_id()
    }

    pub fn try_decode<T: AnyEvent>(&self) -> Option<T> {
        self.is::<T>()
            .then(|| deserialize(&self.data).expect("Failed to deserialize argument"))
    }
}

pub trait AnyEvent: Serialize + DeserializeOwned {
    fn unique_id() -> &'static str;
}
