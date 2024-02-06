extern crate proc_macro;

use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, Type};
use syn::spanned::Spanned;

#[proc_macro]
pub fn generate_impl(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as Type);

    let type_as_tokens = quote! { #input }; // Convert the type to tokens.
    let type_as_string = format!("{}", type_as_tokens); // Convert the tokens to a string.
    let func_name = format!("sea_nd_{}", type_as_string);
    let func_ident = syn::Ident::new(&func_name, input.span());

    let expanded = quote! {
        impl Arbitrary for #input {
            #[inline(always)]
            fn any() -> #input {
                unsafe { #func_ident() }
            }
        }
    };

    TokenStream::from(expanded)
}
