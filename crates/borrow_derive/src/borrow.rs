use quote::quote;
use syn::{DeriveInput, Field, Fields, Index};

pub fn impl_borrow(input: DeriveInput) -> proc_macro::TokenStream {
    let data = input.data;
    let data = match data {
        syn::Data::Struct(struct_data) => struct_data,
        _ => panic!("Borrow may only be derived for structs"),
    };

    let struct_ident = input.ident;
    let struct_generics = input.generics;

    let (fields, is_named) = match data.fields {
        Fields::Named(named_fields) => (named_fields.named, true),
        Fields::Unnamed(unnamed_fields) => (unnamed_fields.unnamed, false),
        _ => panic!("Borrow may only be derived for structs with named fields"),
    };

    let should_derive_field = |field: &Field| {
        field
            .attrs
            .iter()
            .any(|attr| attr.path.segments.last().unwrap().ident == "borrow")
    };

    let fields = if fields.iter().any(should_derive_field) {
        fields.into_iter().filter(should_derive_field).collect()
    } else {
        fields
    };

    let tokens: proc_macro2::TokenStream = fields
        .into_iter()
        .enumerate()
        .map(|(i, field)| {
            let field_tokens = if is_named {
                let field_ident = field.ident.unwrap();
                quote! {
                    self.#field_ident
                }
            } else {
                let index = Index::from(i);
                quote! {
                    self.#index
                }
            };

            let field_type = field.ty;

            quote! {
                impl #struct_generics std::borrow::Borrow<#field_type> for #struct_ident #struct_generics {
                    fn borrow(&self) -> &#field_type {
                        &#field_tokens
                    }
                }
            }
        })
        .collect();

    tokens.into()
}
