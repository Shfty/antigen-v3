use quote::quote;
use syn::ItemStruct;

use crate::RowInput;

pub fn impl_insert(input: ItemStruct) -> proc_macro::TokenStream {
    let RowInput {
        ident,
        generics,
        where_predicates,
        all_view_names: _,
        concrete_view_names,
        concrete_view_names_plural,
        concrete_view_tys,
        concrete_view_inner_tys,
        option_view_names,
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

    let insert_ty = quote! { (#(#concrete_view_inner_tys,)* #(Option<#option_view_inner_tys>,)*) };

    let tokens = quote! {
        #[async_trait::async_trait]
        impl<#(#generics,)* Table> deebs::Insert<Table> for #ident<#(#generics,)*>
        where
            #(
                #generic_types: Send + Sync + #generic_lt,
            )*
            Table: #(deebs::BorrowColumn<#concrete_view_inner_tys> +)* #(deebs::BorrowColumn<#option_view_inner_tys> +)* Send + Sync,
            #(
                #where_predicates,
            )*
        {
            type Insert = #insert_ty;

            async fn insert(table: &Table, key: deebs::Key, (#(#concrete_view_names,)* #(#option_view_names,)*): #insert_ty) where Table: deebs::Table {
                {
                    futures::join!(
                        #(
                            table.insert::<#concrete_view_inner_tys>(key, #concrete_view_names),
                        )*
                    );

                    futures::join!(
                        #(
                            async move {
                                if let Some(#option_view_names) = #option_view_names {
                                    table.insert::<#option_view_inner_tys>(key, #option_view_names.into()).await;
                                }
                            },
                        )*
                    );
                }

                deebs::Table::update_views(table, &<#ident<#(#generics),*> as deebs::Row<Table>>::inner_types()).await;
            }

            async fn insert_auto(table: &Table, (#(#concrete_view_names,)* #(#option_view_names,)*): #insert_ty) -> deebs::Key where Table: deebs::Table + std::borrow::Borrow<std::sync::atomic::AtomicUsize> {
                let key = table.next_key();
                {
                    futures::join!(
                        #(
                            table.insert::<#concrete_view_inner_tys>(key, #concrete_view_names),
                        )*
                    );

                    futures::join!(
                        #(
                            async move {
                                if let Some(#option_view_names) = #option_view_names {
                                    table.insert::<#option_view_inner_tys>(key, #option_view_names.into()).await;
                                }
                            },
                        )*
                    );
                }

                deebs::Table::update_views(table, &<#ident<#(#generics),*> as deebs::Row<Table>>::inner_types()).await;
                key
            }

            async fn insert_multi<RowIterator>(table: &Table, rows: RowIterator)
                where
                    Table: deebs::Table,
                    RowIterator: Iterator<Item = (deebs::Key, #insert_ty)> + Send
            {
                {
                    let (lower, upper) = rows.size_hint();
                    let length = upper.unwrap_or(lower);

                    #(
                        let mut #concrete_view_names_plural = deebs::WriteColumn::<#concrete_view_inner_tys>::new(std::ops::Deref::deref(&table)).await;
                        #concrete_view_names_plural.reserve(length);
                    )*

                    #(
                        let mut #option_view_names_plural = deebs::WriteColumn::<#option_view_inner_tys>::new(std::ops::Deref::deref(&table)).await;
                    )*

                    let mut rows = async_std::stream::from_iter(rows);
                    while let Some((key, (#(#concrete_view_names,)* #(#option_view_names,)*))) = futures::StreamExt::next(&mut rows).await {
                        #(
                            #concrete_view_names_plural.insert(key, #concrete_view_names.into());
                        )*

                        #(
                            if let Some(#option_view_names) = #option_view_names {
                                #option_view_names_plural.insert(key, #option_view_names.into());
                            }
                        )*
                    }
                }

                deebs::Table::update_views(table, &<#ident<#(#generics),*> as deebs::Row<Table>>::inner_types()).await;
            }

            async fn insert_auto_multi<RowIterator>(table: &Table, rows: RowIterator) -> Vec<deebs::Key>
                where
                    Table: deebs::Table + std::borrow::Borrow<std::sync::atomic::AtomicUsize>,
                    RowIterator: Iterator<Item = #insert_ty> + Send
            {
                let mut keys = vec![];

                {
                    let (lower, upper) = rows.size_hint();
                    let length = upper.unwrap_or(lower);

                    #(
                        let mut #concrete_view_names_plural = deebs::WriteColumn::<#concrete_view_inner_tys>::new(std::ops::Deref::deref(&table)).await;
                        #concrete_view_names_plural.reserve(length);
                    )*

                    #(
                        let mut #option_view_names_plural = deebs::WriteColumn::<#option_view_inner_tys>::new(std::ops::Deref::deref(&table)).await;
                    )*

                    let mut rows = async_std::stream::from_iter(rows);
                    while let Some((#(#concrete_view_names,)* #(#option_view_names,)*)) = futures::StreamExt::next(&mut rows).await {
                        let key = table.next_key();

                        #(
                            #concrete_view_names_plural.insert(key, #concrete_view_names.into());
                        )*

                        #(
                            if let Some(#option_view_names) = #option_view_names {
                                #option_view_names_plural.insert(key, #option_view_names.into());
                            }
                        )*

                        keys.push(key)
                    }
                }

                deebs::Table::update_views(table, &<#ident<#(#generics),*> as deebs::Row<Table>>::inner_types()).await;
                keys
            }
        }
    };

    tokens.into()
}
