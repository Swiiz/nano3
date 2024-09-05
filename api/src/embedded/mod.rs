use serde::Serialize;

#[doc(hidden)]
pub mod __priv {
    #[link(wasm_import_module = "host")]
    extern "C" {
        pub fn print(ptr: u32, len: u32);
    }
}

/// Allows for wasm modules to call extern host functions
///
/// **Warning**: Should be used with caution, not for general use.
pub fn __wasm_call_extern<T: Serialize>(extern_fn: unsafe extern "C" fn(u32, u32), t: T) {
    let bytes = postcard::to_allocvec(&t).unwrap().into_boxed_slice();
    unsafe {
        extern_fn(bytes.as_ptr() as _, bytes.len() as _);
    }
}

/// Port of the [`println!`](https://doc.rust-lang.org/std/macro.println.html) macro from the standard library to wasm.
#[macro_export]
macro_rules! println {
    ($($arg:tt)*) => {
        {
          use $crate::{embedded::{__wasm_call_extern, __priv}, alloc::format};
          let value = format!("{}\n", format!($($arg)*));
          __wasm_call_extern(__priv::print, value);
        }
    }
}
