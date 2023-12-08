use quote::quote;
use syn::ItemStruct;

use crate::RowInput;

pub fn impl_map(input: ItemStruct) -> proc_macro::TokenStream {
    let RowInput {
        ident,
        generics,
        where_predicates,
        all_view_names: _,
        concrete_view_names,
        concrete_view_names_plural: _,
        concrete_view_tys,
        concrete_view_inner_tys,
        option_view_names,
        option_view_names_plural: _,
        option_view_tys,
        option_view_inner_tys,
    } = RowInput::new(input);

    assert!(
        !concrete_view_tys.is_empty() || !option_view_tys.is_empty(),
        "Row struct must have at least one view member."
    );

    let _generic_lt = &generics[0];

    let ty_count = concrete_view_inner_tys.len() + option_view_inner_tys.len();

    let tokens = quote! {
        impl<#(#generics,)* M, O> deebs::Map<M> for #ident<#(#generics,)*>
        where
            #(
                M: for<'any > deebs::Mapper<&'any #concrete_view_inner_tys, Mapped = O>,
            )*
            #(
                M: for<'any> deebs::Mapper<&'any #option_view_inner_tys, Mapped = O>,
            )*
            #(
                #where_predicates,
            )*
        {
            type Iter = std::array::IntoIter<Option<O>, #ty_count>;
            type Item = O;

            fn map(&self) -> Self::Iter {
                let #ident {
                    #(#concrete_view_names,)*
                    #(#option_view_names,)*
                } = self;

                std::array::IntoIter::new([
                    #(
                        Some(M::map(#concrete_view_names.deref())),
                    )*
                    #(
                        #option_view_names.as_deref().map(|#option_view_names| M::map(#option_view_names)),
                    )*
                ])
            }
        }
    };

    tokens.into()
}
