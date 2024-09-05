use proc_macro::TokenStream;
use proc_macro2::{Ident, Span};
use quote::quote;
use syn::{Item, ItemFn};

#[proc_macro_attribute]
pub fn wasm_export(_attr: TokenStream, tokens: TokenStream) -> TokenStream {
    let tokens2 = proc_macro2::TokenStream::from(tokens);
    let parse2 = syn::parse2::<Item>(tokens2).expect("Failed to parse tokens");
    match parse2 {
        Item::Fn(func) => handle_func(func),
        _ => panic!("Only functions are currently supported"),
    }
}

fn handle_func(func: ItemFn) -> TokenStream {
    // Check and make sure our function takes
    // only one argument and panic if not
    if func.sig.inputs.len() != 1 {
        panic!("fns marked with export_wasm can only take 1 argument");
    }
    // Copy this function's identifier
    let ident = func.sig.ident.clone();
    // Create a new identifier with a underscore in front of
    // the original identifier
    let shadows_ident = Ident::new(&format!("{}", ident), Span::call_site());
    // Generate some code with the original and new
    // shadowed function
    let ret = quote! {

        #[no_mangle]
        pub fn #shadows_ident(ptr: i32, len: u32) -> i32 {
            let value = unsafe {
                ::std::slice::from_raw_parts(ptr as _, len as _)
            };
            let arg = deserialize(value).expect("Failed to deserialize argument");
            let ret = #ident(arg);
            let bytes = serialize(&ret).expect("Failed to serialize return value");
            let len = bytes.len();
            unsafe {
                ::std::ptr::write(1 as _, len);
            }
            bytes.as_ptr()
        }
    };
    ret.into()
}
