#![no_std]

use nano_api::{derive::on_event, event::RawEvent, println, OnStart};

#[on_event]
pub fn handle_event(event: &RawEvent) {
    println!("WASM: {}", event.unique_id);
    if let Some(start) = event.try_decode::<OnStart>() {
        println!("Example module started from host: {}", start.host);
    }
}
