use quote::quote;
use syn::ItemStruct;

use crate::RowInput;

pub fn impl_row(input: ItemStruct) -> proc_macro::TokenStream {
    let RowInput {
        ident,
        generics,
        where_predicates,
        all_view_names,
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

    let generic_lt = &generics[0];
    let generic_types = &generics[1..];

    let _insert_ty = quote! { (#(#concrete_view_inner_tys,)* #(Option<#option_view_inner_tys>,)*) };

    let tokens = quote! {
        #[async_trait::async_trait]
        impl<#(#generics,)* Table> deebs::Row<#generic_lt, Table> for #ident<#(#generics,)*>
        where
            #(
                #generic_types: Send + Sync + #generic_lt,
            )*
            Table: #(deebs::BorrowColumn<#concrete_view_inner_tys> +)* #(deebs::BorrowColumn<#option_view_inner_tys> +)* Send + Sync,
            #(
                #where_predicates,
            )*
        {
            const HEADER: &'static [&'static str] = &[#(stringify!(#concrete_view_inner_tys),)* #(stringify!(#option_view_inner_tys),)*];

            fn inner_types() -> Vec<std::any::TypeId> {
                vec![#(std::any::TypeId::of::<#concrete_view_inner_tys>(),)* #(std::any::TypeId::of::<#option_view_inner_tys>(),)*]
            }

            async fn new(table: &#generic_lt Table, key: &deebs::Key) -> Self {
                let (#(#concrete_view_names,)*) = futures::join!(#(deebs::#concrete_view_tys::<#concrete_view_inner_tys>::new(table, key),)*);
                let (#(#option_view_names,)*) = futures::join!(#(deebs::#option_view_tys::<#option_view_inner_tys>::new(table, key),)*);

                let (#(#concrete_view_names,)*) = (#(#concrete_view_names.unwrap_or_else(|| panic!("{:?} has no {} cell.", key, stringify!(#concrete_view_names))),)*);

                #ident { #(#all_view_names,)* }
            }
        }
    };

    tokens.into()
}
