#![recursion_limit = "128"]

extern crate proc_macro;

mod attr;
mod bound;
mod common;
mod de;
mod ser;

use proc_macro::TokenStream;
use syn::{parse_macro_input, DeriveInput};

type DeriveResult<T> = std::result::Result<T, T>;

#[proc_macro_derive(Serialize, attributes(toctoc))]
pub fn derive_serialize(input: TokenStream) -> TokenStream {
    ser::derive(parse_macro_input!(input as DeriveInput))
        .unwrap_or_else(|err| err)
        .into()
}

#[proc_macro_derive(Deserialize, attributes(toctoc))]
pub fn derive_deserialize(input: TokenStream) -> TokenStream {
    de::derive(parse_macro_input!(input as DeriveInput))
        .unwrap_or_else(|err| err.to_compile_error())
        .into()
}
