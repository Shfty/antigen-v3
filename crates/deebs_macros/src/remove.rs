use quote::quote;
use syn::ItemStruct;

use crate::RowInput;

pub fn impl_remove(input: ItemStruct) -> proc_macro::TokenStream {
    let RowInput {
        ident,
        generics,
        where_predicates,
        all_view_names: _,
        concrete_view_names: _,
        concrete_view_names_plural: _,
        concrete_view_tys,
        concrete_view_inner_tys,
        option_view_names: _,
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
        impl<#(#generics,)* Table> deebs::Remove<Table> for #ident<#(#generics,)*>
        where
            #(
                #generic_types: Send + Sync + #generic_lt,
            )*
            Table: #(deebs::BorrowColumn<#concrete_view_inner_tys> +)* #(deebs::BorrowColumn<#option_view_inner_tys> +)* Send + Sync,
            #(
                #where_predicates,
            )*
        {
            async fn remove(table: &Table, key: deebs::Key) where Table: deebs::Table {
                {
                    futures::join!(
                        #(
                            table.remove::<#concrete_view_inner_tys>(key),
                        )*
                        #(
                            table.remove::<#option_view_inner_tys>(key),
                        )*
                    );
                }

                deebs::Table::update_views(table, &<#ident<#(#generics),*> as deebs::Row<Table>>::inner_types()).await;
            }

            async fn remove_multi<S>(table: &Table, keys: S) where Table: deebs::Table, S: futures::Stream<Item = deebs::Key> + Send {
                {
                    futures::pin_mut!(keys);
                    while let Some(key) = futures::StreamExt::next(&mut keys).await {
                        futures::join!(
                            #(
                                table.remove::<#concrete_view_inner_tys>(key),
                            )*
                            #(
                                table.remove::<#option_view_inner_tys>(key),
                            )*
                        );
                    }
                }

                deebs::Table::update_views(table, &<#ident<#(#generics),*> as deebs::Row<Table>>::inner_types()).await;
            }
        }
    };

    tokens.into()
}
