extern crate proc_macro;
extern crate syn;
#[macro_use]
extern crate quote;

use proc_macro::TokenStream;
use syn::*;
type SynStream = syn::export::TokenStream2;


#[proc_macro_derive(XML, attributes(attr))]
pub fn object(input: TokenStream) -> TokenStream {
    let ast = parse_macro_input!(input as DeriveInput);

    // Build the impl
    impl_object(&ast)
}

struct FieldAttrs {
    // attribute name
    name: String,

    // default value if attribute is not set
    default: Option<Option<String>>,

}

