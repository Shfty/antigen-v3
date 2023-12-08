use quote::quote;
use syn::{GenericArgument, Ident, ItemStruct, PathArguments, Type};

pub fn impl_table(input: ItemStruct) -> proc_macro::TokenStream {
    let ident = input.ident;
    let generics = input.generics;

    let mut view_inner_tys: Vec<Type> = vec![];
    let mut view_idents: Vec<Ident> = vec![];

    for field in input.fields {
        if let Type::Path(path) = field.ty {
            let first = path
                .path
                .segments
                .first()
                .expect("Path must have a first segment");

            if first.ident == "View" {
                if let PathArguments::AngleBracketed(args) = &first.arguments {
                    if let GenericArgument::Type(ty) = &args.args[0] {
                        let view_inner_ty = ty.clone();
                        view_inner_tys.push(view_inner_ty);
                        view_idents.push(field.ident.expect("Field must have an ident."));
                    } else {
                        panic!("First generic argument is not a type.")
                    }
                } else {
                    panic!("Path arguments must be angle-bracketed.")
                }
            }
        } else {
            panic!("All fields must be paths.")
        }
    }

    let tokens = quote! {
        #[async_trait::async_trait]
        impl #generics deebs::Table for #ident #generics {
            type Key = deebs::Key;
            
            async fn update_views(&self, type_ids: &[std::any::TypeId]) {
                #(
                    if <#view_inner_tys as deebs::Row<Self>>::inner_types()
                        .iter()
                        .any(|ty| type_ids.contains(ty))
                    {
                        self.#view_idents.update(self).await;
                    }
                )*
            }
        }
    };

    tokens.into()
}
