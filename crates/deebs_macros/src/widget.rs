use quote::quote;
use syn::ItemStruct;

pub fn impl_widget(input: ItemStruct) -> proc_macro::TokenStream {
    let ident = input.ident;
    
    let tokens = quote! {
        impl egui::Widget for &#ident {
            fn ui(self, ui: &mut egui::Ui) -> egui::Response {
                ui.label(self.to_string())
            }
        }
    };

    tokens.into()
}
