extern crate proc_macro;

use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, DeriveInput};

#[proc_macro_derive(StructName)]
pub fn derive_struct_name(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let ident = input.ident;

    let expanded = quote! {
        impl StructName for #ident {
            fn struct_name(&self) -> &'static str {
                stringify!(#ident)
            }
        }
    };

    TokenStream::from(expanded)
}
