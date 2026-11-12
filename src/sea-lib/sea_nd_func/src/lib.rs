extern crate proc_macro;

use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, Data, DeriveInput, Fields, Type};
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

// Build an expression that constructs `path` with every field nondet.
fn nondet_fields(path: proc_macro2::TokenStream, fields: &Fields) -> proc_macro2::TokenStream {
    match fields {
        Fields::Named(named) => {
            let names = named.named.iter().map(|f| &f.ident);
            quote! { #path { #(#names: ::sea::Arbitrary::any()),* } }
        }
        Fields::Unnamed(unnamed) => {
            let anys = unnamed.unnamed.iter().map(|_| quote! { ::sea::Arbitrary::any() });
            quote! { #path ( #(#anys),* ) }
        }
        Fields::Unit => quote! { #path },
    }
}

/// `#[derive(Arbitrary)]`: a struct gets every field nondet; an enum picks a
/// nondet variant (and nondet fields for it). Generic parameters must
/// implement `Arbitrary`.
#[proc_macro_derive(Arbitrary)]
pub fn derive_arbitrary(input: TokenStream) -> TokenStream {
    let mut input = parse_macro_input!(input as DeriveInput);
    let name = &input.ident;

    for param in input.generics.type_params_mut() {
        param.bounds.push(syn::parse_quote!(::sea::Arbitrary));
    }
    let (impl_generics, ty_generics, where_clause) = input.generics.split_for_impl();

    let body = match &input.data {
        Data::Struct(data) => nondet_fields(quote! { Self }, &data.fields),
        Data::Enum(data) => {
            let n = data.variants.len() as u32;
            if n == 0 {
                return syn::Error::new(name.span(), "cannot derive Arbitrary for an empty enum")
                    .to_compile_error()
                    .into();
            }
            // The last variant is the `_` arm, so the match is exhaustive
            // without an unreachable!() (which would be a reachable panic).
            let arms = data.variants.iter().enumerate().map(|(i, v)| {
                let vname = &v.ident;
                let ctor = nondet_fields(quote! { Self::#vname }, &v.fields);
                if i as u32 == n - 1 {
                    quote! { _ => #ctor, }
                } else {
                    let i = i as u32;
                    quote! { #i => #ctor, }
                }
            });
            quote! {
                let tag: u32 = ::sea::Arbitrary::any();
                ::sea::assume(tag < #n);
                match tag { #(#arms)* }
            }
        }
        Data::Union(_) => {
            return syn::Error::new(name.span(), "cannot derive Arbitrary for a union")
                .to_compile_error()
                .into();
        }
    };

    let expanded = quote! {
        impl #impl_generics ::sea::Arbitrary for #name #ty_generics #where_clause {
            #[inline(always)]
            fn any() -> Self {
                #body
            }
        }
    };
    TokenStream::from(expanded)
}
