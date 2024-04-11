extern crate proc_macro;

use proc_macro::TokenStream;
use quote::__private::Span;
use quote::quote;
use syn::{parse_macro_input, ItemTrait, TraitItem, Ident, parse_quote, TraitItemMethod};

fn generate_attr_names(method: &TraitItemMethod, prefixes: &[&str]) -> Vec<Ident> {
    prefixes
        .iter()
        .map(|prefix| Ident::new(&format!("{}_{}", prefix, &method.sig.ident), method.sig.ident.span()))
        .collect()
}

#[proc_macro_attribute]
pub fn seamock(_args: TokenStream, input: TokenStream) -> TokenStream {
    // Parse the input trait
    let input = parse_macro_input!(input as ItemTrait);

    let mock_struct_name = Ident::new(&format!("Mock{}", &input.ident), Span::call_site());

    let trait_methods = input.items.iter().filter_map(|item| {
        if let TraitItem::Method(method) = item {
            Some(method)
        } else {
            None
        }
    });

    let max_times = trait_methods.clone().flat_map(|method| {
        generate_attr_names(method, &["max_times"])
    });

    let times = trait_methods.clone().flat_map(|method| {
        generate_attr_names(method, &["times"])
    });

    let mut returning_attrs = vec!{};
    let mut with_attrs = vec!{};
    let mut with_methods = vec!{};

    let ret = trait_methods.clone().flat_map(|method| {
        let method_output = &method.sig.output;
        let mut method_inputs = method.sig.inputs.clone();
        let mut val_with_params = vec![];
        // Check if the first argument is `&self` and remove it
        if let Some(first_arg) = method_inputs.first() {
            if let syn::FnArg::Receiver(_) = first_arg {
                method_inputs = method_inputs.iter().skip(1).cloned().collect();
            }
        }
        // For each argument, create WithVal<T> where T is the argument type
        for arg in method_inputs.iter() {
            if let syn::FnArg::Typed(pat_type) = arg {
                let arg_type = &pat_type.ty;
                let with_val_type: syn::Type = parse_quote! { WithVal<#arg_type> };
                val_with_params.push(with_val_type);
            }
        }
        // Create a tuple of WithVal<T> for each argument
        let val_with_tuple = if val_with_params.is_empty() {
            None
        } else {
            Some(quote! { ( #(#val_with_params),* ) })
        };
        let returning_attr = Ident::new(&format!("val_returning_{}", &method.sig.ident), method.sig.ident.span());
        let with_attr = Ident::new(&format!("val_with_{}", &method.sig.ident), method.sig.ident.span());
        let with_method = Ident::new(&format!("with_{}", &method.sig.ident), method.sig.ident.span());
        returning_attrs.push(quote! {
            #returning_attr: |#method_inputs| Default::default(),
        });
        if val_with_tuple.is_some() {
            with_attrs.push(quote! {
                #with_attr: None,
            });
            with_methods.push(quote! {
                fn #with_method(&mut self, with: #val_with_tuple) -> &mut Self {
                    self.#with_attr = Some(with);
                    self
                }
            });
            quote! {
                #returning_attr: fn(#method_inputs) #method_output,
                #with_attr: Option<#val_with_tuple>,
            }
        } else {
            quote! {
                #returning_attr: fn(#method_inputs) #method_output,
            }
        }
    });

    let ret_methods = trait_methods.clone().flat_map(|method| {
        let method_output = &method.sig.output;
        let mut method_inputs = method.sig.inputs.clone();
        // Remove `&self`
        if let Some(first_arg) = method_inputs.first() {
            if let syn::FnArg::Receiver(_) = first_arg {
                method_inputs = method_inputs.iter().skip(1).cloned().collect();
            }
        }
        let input_method = Ident::new(&format!("returning_{}", &method.sig.ident), method.sig.ident.span());
        let update_attr = Ident::new(&format!("val_returning_{}", &method.sig.ident), method.sig.ident.span());
        Some (quote! {
            fn #input_method(&mut self, f: fn(#method_inputs) #method_output) -> &mut Self {
                self.#update_attr = f;
                self
            }
        })
    });

    let times_methods = trait_methods.clone().flat_map(|method| {
        let times_method = Ident::new(&format!("times_{}", &method.sig.ident), method.sig.ident.span());
        let update_attr = Ident::new(&format!("max_times_{}", &method.sig.ident), method.sig.ident.span());
        Some (quote! {
            fn #times_method(&mut self, val: u64) -> &mut Self {
                self.#update_attr = val;
                self
            }
        })
    });

    let expect_times_methods = trait_methods.clone().flat_map(|method| {
        let times_method = Ident::new(&format!("expect_times_{}", &method.sig.ident), method.sig.ident.span());
        let times_attr = Ident::new(&format!("times_{}", &method.sig.ident), method.sig.ident.span());
        Some (quote! {
            fn #times_method(&mut self, val: u64) -> bool {
                *self.#times_attr.borrow() == val
            }
        })
    });

    let methods_impl = trait_methods.clone().flat_map(|method| {
        let method_name =  &method.sig.ident;
        let method_output = &method.sig.output;
        let method_inputs = &method.sig.inputs;
        let mut params = vec!{};
        // For each argument, create WithVal<T> where T is the argument type
        for arg in method_inputs.iter() {
            if let syn::FnArg::Typed(pat_type) = arg {
                let arg_name = match &*pat_type.pat {
                    syn::Pat::Ident(ident) => ident.ident.clone(),
                    _ => unimplemented!(), // Handle other patterns as needed
                };
                params.push(quote! { #arg_name, });
            }
        }

        let times_attr = Ident::new(&format!("times_{}", &method.sig.ident), method.sig.ident.span());
        let max_times_attr = Ident::new(&format!("max_times_{}", &method.sig.ident), method.sig.ident.span());
        let ret_func = Ident::new(&format!("val_returning_{}", &method.sig.ident), method.sig.ident.span());
        let error = format!("Hit times limit for {}", &method.sig.ident);

        Some (quote! {
            fn #method_name(#method_inputs) #method_output {
                self.#times_attr.replace_with(|&mut old| old + 1);
                if (*self.#times_attr.borrow() > self.#max_times_attr) {
                    sea::sea_printf!(#error, self.#max_times_attr);
                    verifier::vassert!(false);
                }
                (self.#ret_func)(#(#params)*)
            }
        })
    });

    let times_clone = times.clone();
    let max_times_clone = max_times.clone();

    // Generate the Mock struct
    let mock_struct = quote! {
        struct #mock_struct_name {
            #(
                #max_times_clone: u64,
            )*
            #(
                #times_clone: RefCell<u64>,
            )*
            #(
                #ret
            )*
        }
    };

    let times_clone = times.clone();
    let max_times_clone = max_times.clone();

    // Implement the trait for MockContext
    let trait_original = &input.ident;
    let mock_impl = quote! {
        impl #mock_struct_name {
            pub fn new() -> Self {
                Self {
                    #(
                        #max_times_clone: u64::MAX,
                    )*
                    #(
                        #times_clone: RefCell::new(0),
                    )*
                    #(
                        #returning_attrs
                    )*
                    #(
                        #with_attrs
                    )*
                }
            }
            #(#ret_methods)*
            #(#with_methods)*
            #(#times_methods)*
            #(#expect_times_methods)*
        }
    };

    let trait_impl = quote! {
        impl #trait_original for #mock_struct_name {
            #(#methods_impl)*
        }
    };

    // Combine the generated tokens
    let expanded = quote! {
        use core::cell::RefCell;
        enum WithVal<T> {
            Gt(T),
            Gte(T),
            Lt(T),
            Lte(T),
            Eq(T),
        }
        #input
        #mock_struct
        #mock_impl
        #trait_impl
    };


    TokenStream::from(expanded)
}
