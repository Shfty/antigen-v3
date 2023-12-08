use proc_macro2::Span;
use syn::{GenericArgument, GenericParam, Ident, ItemStruct, PathArguments, Type, WherePredicate};

mod common_keys;
mod insert;
mod map;
mod remove;
mod row;
mod table;
mod widget;
mod widgets;

pub(crate) struct RowInput {
    ident: Ident,
    generics: Vec<GenericParam>,
    where_predicates: Vec<WherePredicate>,
    all_view_names: Vec<Ident>,

    concrete_view_names: Vec<Ident>,
    concrete_view_names_plural: Vec<Ident>,
    concrete_view_tys: Vec<Ident>,
    concrete_view_inner_tys: Vec<Type>,

    option_view_names: Vec<Ident>,
    option_view_names_plural: Vec<Ident>,
    option_view_tys: Vec<Ident>,
    option_view_inner_tys: Vec<Type>,
}

impl RowInput {
    pub fn new(input: ItemStruct) -> Self {
        let ident = input.ident;
        let generics = input.generics;

        if let syn::GenericParam::Lifetime(_) = generics.params.first().unwrap() {
        } else {
            panic!("First generic param must be a lifetime.")
        };

        let generics_vec = generics.params.into_iter().collect::<Vec<_>>();
        let where_predicates_vec = if let Some(where_clause) = generics.where_clause {
            where_clause.predicates.into_iter().collect::<Vec<_>>()
        } else {
            vec![]
        };

        let mut all_view_names: Vec<Ident> = vec![];

        let mut concrete_view_names: Vec<Ident> = vec![];
        let mut concrete_view_names_plural: Vec<Ident> = vec![];
        let mut concrete_view_tys: Vec<Ident> = vec![];
        let mut concrete_view_inner_tys: Vec<Type> = vec![];

        let mut option_view_names: Vec<Ident> = vec![];
        let mut option_view_names_plural: Vec<Ident> = vec![];
        let mut option_view_tys: Vec<Ident> = vec![];
        let mut option_view_inner_tys: Vec<Type> = vec![];

        for field in input.fields {
            let field_ident = field.ident.expect("All fields must have an ident");

            all_view_names.push(field_ident.clone());

            if let Ok(inner_type) = parse_option_type(&field.ty) {
                option_view_names.push(field_ident.clone());
                option_view_names_plural.push(Ident::new(
                    &(field_ident.to_string() + "_collection"),
                    Span::call_site(),
                ));
                let (type_ident, ty) = parse_cell_view_type(&inner_type).expect("Unexpected type.");
                option_view_tys.push(type_ident);
                option_view_inner_tys.push(ty);
            } else {
                concrete_view_names.push(field_ident.clone());
                concrete_view_names_plural.push(Ident::new(
                    &(field_ident.to_string() + "_collection"),
                    Span::call_site(),
                ));
                let (ident, ty) = parse_cell_view_type(&field.ty).expect("Unexpected type.");
                concrete_view_tys.push(ident);
                concrete_view_inner_tys.push(ty);
            }
        }

        RowInput {
            ident,
            generics: generics_vec,
            where_predicates: where_predicates_vec,
            all_view_names,
            concrete_view_names,
            concrete_view_names_plural,
            concrete_view_tys,
            concrete_view_inner_tys,
            option_view_names,
            option_view_names_plural,
            option_view_tys,
            option_view_inner_tys,
        }
    }
}

pub(crate) fn parse_cell_view_type(ty: &Type) -> Result<(Ident, Type), &str> {
    if let Type::Path(path) = ty {
        let first = path
            .path
            .segments
            .first()
            .expect("Path must have a first segment");

        assert!(first.ident == "ReadCell" || first.ident == "WriteCell");
        let view_ty = first.ident.clone();

        if let PathArguments::AngleBracketed(args) = &first.arguments {
            let lt = &args.args[0];

            if let GenericArgument::Lifetime(_) = lt {
            } else {
                return Err("First generic argument is not a lifetime.");
            }

            if let GenericArgument::Type(ty) = &args.args[1] {
                let view_inner_ty = ty.clone();
                Ok((view_ty, view_inner_ty))
            } else {
                Err("Second generic argument is not a type.")
            }
        } else {
            Err("Path arguments must be angle-bracketed.")
        }
    } else {
        Err("All fields must be paths.")
    }
}

pub(crate) fn parse_option_type(ty: &Type) -> Result<Type, &str> {
    if let Type::Path(path) = ty {
        let first = path
            .path
            .segments
            .first()
            .expect("Path must have a first segment");

        if first.ident == "Option" {
            if let PathArguments::AngleBracketed(args) = &first.arguments {
                let arg = &args.args[0];
                if let GenericArgument::Type(ty) = arg {
                    Ok(ty.clone())
                } else {
                    Err("Unrecognized Option type.")
                }
            } else {
                Err("Option arguments must be angle-bracketed.")
            }
        } else {
            Err("Unrecognized type.")
        }
    } else {
        Err("All fields must be paths.")
    }
}

#[proc_macro_derive(Row)]
pub fn derive_row(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let input = syn::parse_macro_input!(input);
    row::impl_row(input)
}

#[proc_macro_derive(Insert)]
pub fn derive_insert(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let input = syn::parse_macro_input!(input);
    insert::impl_insert(input)
}

#[proc_macro_derive(Remove)]
pub fn derive_remove(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let input = syn::parse_macro_input!(input);
    remove::impl_remove(input)
}

#[proc_macro_derive(CommonKeys)]
pub fn derive_common_keys(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let input = syn::parse_macro_input!(input);
    common_keys::impl_common_keys(input)
}

#[proc_macro_derive(Map)]
pub fn derive_map(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let input = syn::parse_macro_input!(input);
    map::impl_map(input)
}

#[proc_macro_derive(Widget)]
pub fn derive_widget(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let input = syn::parse_macro_input!(input);
    widget::impl_widget(input)
}

#[proc_macro_derive(Widgets)]
pub fn derive_widgets(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let input = syn::parse_macro_input!(input);
    widgets::impl_widgets(input)
}

#[proc_macro_derive(Table)]
pub fn derive_table(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let input = syn::parse_macro_input!(input);
    table::impl_table(input)
}
