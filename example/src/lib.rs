#![no_std]

use nano_api::{derive::wasm_export, deserialize, Event};

#[link(wasm_import_module = "host")]
extern "C" {
    fn hello(param: i32);
}

#[no_mangle]
pub fn _handle_event(len: u32) {
    let value = unsafe { core::slice::from_raw_parts(1 as _, len as _) };
    let event: Event = deserialize(value).expect("Failed to deserialize argument");

    unsafe {
        hello(event.name.len() as i32);
    }
}

pub struct Plugin {}

impl Plugin {}
