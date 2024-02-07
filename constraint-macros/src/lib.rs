mod constraints;

const REG_BITS: usize = 32;
const SHAMT_BITS: usize = 5;

extern crate proc_macro;
use proc_macro::TokenStream;

use constraints::gen::generate as generate_constraints;
use syn::parse_macro_input;

#[proc_macro]
pub fn air(item: TokenStream) -> TokenStream {
    let config = parse_macro_input!(item);
    generate_constraints(config)
}
