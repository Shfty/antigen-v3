//! Automatically derive [`Borrow`] for the fields of a given struct

mod borrow;
use borrow::impl_borrow;

#[proc_macro_derive(Borrow, attributes(borrow))]
pub fn derive_borrow(tokens: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let input = syn::parse_macro_input!(tokens);
    impl_borrow(input)
}
