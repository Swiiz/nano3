#![no_std]

use nano_api::{derive::on_event, println, Event};

#[on_event]
pub fn handle_event(event: Event) {
    println!("WASM: {}", event.name);
}
