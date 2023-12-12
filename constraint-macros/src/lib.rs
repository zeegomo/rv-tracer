mod constraints;

const REG_BITS: usize = 32;
const REG_NUM_PO2: usize = 5;
const SHAMT_BITS: usize = 5;

extern crate proc_macro;
use proc_macro::TokenStream;

use constraints::gen::generate as generate_constraints;
use constraints::parse::{Air, Constraint, Field};
use syn::parse_macro_input;

#[proc_macro]
pub fn air(item: TokenStream) -> TokenStream {
    let config = parse_macro_input!(item);
    generate_constraints(config)
}
