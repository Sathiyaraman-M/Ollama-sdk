use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, DeriveInput};

#[proc_macro_derive(FromBytes)]
pub fn derive_from_bytes(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let name = input.ident;

    let expanded = quote! {
        impl #name {
            pub fn from_bytes(bytes: ::bytes::Bytes) -> crate::Result<Self> {
                ::serde_json::from_slice(&bytes).map_err(crate::Error::JsonParse)
            }
        }
    };
    TokenStream::from(expanded)
}
