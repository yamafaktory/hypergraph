extern crate proc_macro;
extern crate proc_macro2;
#[macro_use]
extern crate quote;
extern crate syn;

use proc_macro2::TokenStream;

#[proc_macro_attribute]
pub fn add_derive(
    _metadata: proc_macro::TokenStream,
    input: proc_macro::TokenStream,
) -> proc_macro::TokenStream {
    let input: TokenStream = input.into();
    let output = quote! {
        #[derive(Debug, Serialize, Deserialize, etc, ...)]
        #input
    };
    output.into()
}
