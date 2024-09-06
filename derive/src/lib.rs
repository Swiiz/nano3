use proc_macro::TokenStream;
use proc_macro2::{Ident, Span};
use quote::quote;
use syn::{Item, ItemFn};

#[proc_macro_attribute]
pub fn on_event(_attr: TokenStream, tokens: TokenStream) -> TokenStream {
    let tokens2 = proc_macro2::TokenStream::from(tokens);
    let parse2 = syn::parse2::<Item>(tokens2).expect("Failed to parse tokens");
    match parse2 {
        Item::Fn(func) => handle_func(func),
        _ => panic!("Only functions are supported"),
    }
}

fn handle_func(func: ItemFn) -> TokenStream {
    if func.sig.inputs.len() != 1 {
        panic!("fns marked with on_event can only take 1 argument");
    }
    let ident = func.sig.ident.clone();
    let shadows_ident = Ident::new("_handle_event", Span::call_site());
    let ret = quote! {

        #[no_mangle]
        pub fn #shadows_ident(len: u32) {
           let value = unsafe { core::slice::from_raw_parts(1 as _, len as _) };
            let event: ::nano_api::event::RawEvent = ::nano_api::deserialize(value).expect("Failed to deserialize argument");

            #func
            #ident(&event);
        }
    };
    ret.into()
}

#[proc_macro_attribute]
pub fn wasm_event(_attr: TokenStream, input: TokenStream) -> TokenStream {
    let tokens2 = proc_macro2::TokenStream::from(input);
    let parse2 = syn::parse2::<Item>(tokens2.clone()).expect("Failed to parse tokens");

    match parse2 {
        Item::Struct(s) => {
            let ident = s.ident;
            let _crate = if &std::env::var("CARGO_PKG_NAME").unwrap() == "nano_api" {
                Ident::new("crate", Span::call_site())
            } else {
                Ident::new("nano_api", Span::call_site())
            };
            let ret = quote! {
                #[derive(#_crate ::Serialize, #_crate ::Deserialize)]
                #tokens2

                impl #_crate ::event::AnyEvent for #ident {
                    fn unique_id() -> &'static str {
                        concat!(
                            env!("CARGO_PKG_NAME"),
                            "@",
                            module_path!(),
                            "::",
                            stringify!(#ident)
                        )
                    }
                }
            };
            ret.into()
        }
        _ => panic!("Only structs are supported"),
    }
}
