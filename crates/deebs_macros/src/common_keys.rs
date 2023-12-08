use quote::quote;
use syn::ItemStruct;

use crate::RowInput;

pub fn impl_common_keys(input: ItemStruct) -> proc_macro::TokenStream {
    let RowInput {
        ident,
        generics,
        where_predicates,
        all_view_names: _,
        concrete_view_names: _,
        concrete_view_names_plural,
        concrete_view_tys,
        concrete_view_inner_tys,
        option_view_names: _,
        option_view_names_plural,
        option_view_tys,
        option_view_inner_tys,
    } = RowInput::new(input);

    assert!(
        !concrete_view_tys.is_empty() || !option_view_tys.is_empty(),
        "Row struct must have at least one view member."
    );

    let generic_lt = &generics[0];
    let generic_types = &generics[1..];

    let _insert_ty = quote! { (#(#concrete_view_inner_tys,)* #(Option<#option_view_inner_tys>,)*) };

    let tokens = quote! {
        #[async_trait::async_trait]
        impl<#(#generics,)* Table> deebs::CommonKeys<Table> for #ident<#(#generics,)*>
        where
            #(
                #generic_types: Send + Sync + #generic_lt,
            )*
            Table: #(deebs::BorrowColumn<#concrete_view_inner_tys> +)* #(deebs::BorrowColumn<#option_view_inner_tys> +)* Send + Sync,
            #(
                #where_predicates,
            )*
        {
            async fn common_keys(table: &Table) -> async_std::stream::FromIter<std::collections::btree_set::IntoIter<deebs::Key>> {
                let (#(#concrete_view_names_plural,)*) = futures::join!(#(deebs::ReadColumn::<#concrete_view_inner_tys>::new(table),)*);
                let (#(#option_view_names_plural,)*) = futures::join!(#(deebs::ReadColumn::<#option_view_inner_tys>::new(table),)*);

                let mut keys: std::collections::BTreeSet<deebs::Key> = Default::default();

                for key in std::iter::empty()
                #(
                    .chain(#concrete_view_names_plural.keys())
                )*
                #(
                    .chain(#option_view_names_plural.keys())
                )*
                {
                    if keys.contains(key) {
                        continue;
                    }

                    let contains = true;
                    #(
                        let contains = contains & #concrete_view_names_plural.contains_key(key);
                    )*

                    if contains {
                        keys.insert(*key);
                    }
                }

                async_std::stream::from_iter(keys.into_iter())
            }
        }
    };

    tokens.into()
}
