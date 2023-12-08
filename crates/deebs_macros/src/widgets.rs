
use quote::quote;
use syn::{ItemStruct};

use crate::RowInput;

pub fn impl_widgets(input: ItemStruct) -> proc_macro::TokenStream {
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

    let mut concrete_view_names_immut = vec![];
    let mut concrete_view_tys_immut = vec![];
    let mut concrete_view_inner_tys_immut = vec![];

    let mut concrete_view_names_mut = vec![];
    let mut concrete_view_tys_mut = vec![];
    let mut concrete_view_inner_tys_mut = vec![];

    let mut option_view_names_immut = vec![];
    let mut option_view_tys_immut = vec![];
    let mut option_view_inner_tys_immut = vec![];

    let mut option_view_names_mut = vec![];
    let mut option_view_tys_mut = vec![];
    let mut option_view_inner_tys_mut = vec![];

    for (i, ty) in concrete_view_tys.iter().enumerate() {
        match ty.to_string().as_str() {
            "ReadCell" => {
                concrete_view_names_immut.push(&concrete_view_names[i]);
                concrete_view_tys_immut.push(&concrete_view_tys[i]);
                concrete_view_inner_tys_immut.push(&concrete_view_inner_tys[i]);
            }
            "WriteCell" => {
                concrete_view_names_mut.push(&concrete_view_names[i]);
                concrete_view_tys_mut.push(&concrete_view_tys[i]);
                concrete_view_inner_tys_mut.push(&concrete_view_inner_tys[i]);
            }
            _ => panic!("Unrecognized type"),
        }
    }

    for (i, ty) in option_view_tys.iter().enumerate() {
        match ty.to_string().as_str() {
            "ReadCell" => {
                option_view_names_immut.push(&option_view_names[i]);
                option_view_tys_immut.push(&option_view_tys[i]);
                option_view_inner_tys_immut.push(&option_view_inner_tys[i]);
            }
            "WriteCell" => {
                option_view_names_mut.push(&option_view_names[i]);
                option_view_tys_mut.push(&option_view_tys[i]);
                option_view_inner_tys_mut.push(&option_view_inner_tys[i]);
            }
            _ => panic!("Unrecognized type"),
        }
    }

    let tokens = quote! {
        impl<#(#generics,)*> antigen_egui::Widgets for #ident<#(#generics,)*>
        where
        #(
            for<'any> &'any #concrete_view_inner_tys_immut: egui::Widget,
        )*
        #(
            for<'any> &'any mut #concrete_view_inner_tys_mut: egui::Widget,
        )*
        #(
            for<'any> &'any #option_view_inner_tys_immut: egui::Widget,
        )*
        #(
            for<'any> &'any mut #option_view_inner_tys_mut: egui::Widget,
        )*
        #(
            #where_predicates,
        )*
        {
            fn widgets(&mut self, ui: &mut egui::Ui) -> egui::Response {
                use std::ops::Deref;
                use std::ops::DerefMut;

                let response = ui.interact(egui::Rect::NOTHING, egui::Id::new(stringify!(#ident)), egui::Sense::hover());

                #(
                    let response = response.union(ui.add(self.#concrete_view_names_immut.deref()));
                )*

                #(
                    let response = response.union(ui.add(self.#concrete_view_names_mut.deref_mut()));
                )*

                #(
                    let response = response.union(if let Some(#option_view_names_immut) = &self.#option_view_names_immut {
                        ui.add(#option_view_names_immut.deref())
                    } else {
                        ui.label("")
                    });
                )*

                #(
                    let response = response.union(if let Some(#option_view_names_mut) = &mut self.#option_view_names_mut {
                        ui.add(#option_view_names_mut.deref_mut())
                    } else {
                        ui.label("")
                    });
                )*

                response
            }
        }
    };

    tokens.into()
}
