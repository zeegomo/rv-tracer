pub(crate) mod folders;
mod gen;
pub(crate) mod parse;

extern crate proc_macro;
use proc_macro::TokenStream;

use folders::fold_bitwise;
use gen::generate;
use parse::{Air, Constraint, Field};
use syn::parse_macro_input;

#[proc_macro]
pub fn air(item: TokenStream) -> TokenStream {
    let config = parse_macro_input!(item);
    generate(config)
}

#[proc_macro]
pub fn bitwise_air(item: TokenStream) -> TokenStream {
    let config: Air = parse_macro_input!(item);
    let constraints = config
        .constraints
        .clone()
        .into_iter()
        .flat_map(|c| {
            fold_bitwise(c.expr, 32)
                .into_iter()
                .map(move |expr| Constraint {
                    degree: c.degree,
                    expr,
                })
        })
        .collect();
    let config = Air {
        constraints,
        ..config
    };
    generate(config)
}
