use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, DeriveInput};

#[proc_macro_derive(HlsAttrs)]
pub fn derive_hls_attrs(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);

    let expanded = quote! {};

    TokenStream::from(expanded)
}