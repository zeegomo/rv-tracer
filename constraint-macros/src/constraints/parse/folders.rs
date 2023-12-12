use super::{Kind, ShiftConstraint};
use proc_macro2::TokenStream;
use quote::quote;
use syn::{fold::Fold, Expr, ExprPath};
/// Extesions to support nice syntax for constraints.
///
// Replace a constraint of the form rs1 * rs2 = rd with a constraint operating on a single bit
// of rs1, rs2 and rd
// rsx = rsx[..]
struct BitwiseFold {
    bit: usize,
}

impl Fold for BitwiseFold {
    // The only allowed operations are addition, subtraction and multiplication, which are all binary operations
    fn fold_expr(&mut self, mut i: Expr) -> Expr {
        i = replace_reg(i, "rs1", self.bit, "current");
        i = replace_reg(i, "rs2", self.bit, "current");
        i = replace_reg(i, "rd", self.bit, "next");
        i
    }
}

fn replace_variable_with(expr: Expr, find: &str, replace: Expr) -> Expr {
    let find = Expr::Path(ExprPath {
        attrs: vec![],
        qself: None,
        path: syn::parse_str(find).unwrap(),
    });
    if expr == find {
        replace
    } else {
        expr
    }
}

fn replace_reg(original: Expr, reg: &str, bit: usize, base: &str) -> Expr {
    let offset = format!("{}_END", reg.to_uppercase());
    replace_variable_with(
        original,
        reg,
        syn::parse_str(&format!("{base}[{offset}+{bit}]")).unwrap(),
    )
}

// Will output n_bits constraints
pub fn fold_bitwise<const N_BITS: usize>(expr: Expr) -> Vec<Expr> {
    let mut res = Vec::with_capacity(N_BITS);
    let mut fold = BitwiseFold { bit: 0 };
    for i in 0..N_BITS {
        fold.bit = i;
        res.push(fold.fold_expr(expr.clone()));
    }
    res
}

pub fn generate_shift_constraints<const REG_BITS: usize>(c: ShiftConstraint) -> Vec<TokenStream> {
    let mut res = Vec::with_capacity(REG_BITS);
    let src_bits = (0..REG_BITS)
        .map(|i| c.src.to_trace_bit(i))
        .collect::<Vec<_>>();

    match c.kind {
        Kind::Left => {
            for bit in (0..REG_BITS).rev() {
                let dst = c.dst.to_trace_bit(bit);
                res.push(quote! {
                    let src_bits = [#(#src_bits),*];
                    if #bit - shamt >= 0 {
                        let src = src_bits[#bit - shamt];
                        src - #dst
                    } else {
                        #dst
                    }
                })
            }
        }
        Kind::Right => {
            for bit in 0..REG_BITS {
                let dst = c.dst.to_trace_bit(bit);
                res.push(quote! {
                    let src_bits = [#(#src_bits),*];
                    if #bit + shamt < REG_BITS {
                        let src = src_bits[#bit - shamt];
                        src - #dst
                    } else {
                        #dst
                    }
                })
            }
        }
        Kind::RightSticky => {
            for bit in 0..REG_BITS {
                let dst = c.dst.to_trace_bit(bit);
                let fill = c.src.to_trace_bit(REG_BITS - 1);
                res.push(quote! {
                    let src_bits = [#(#src_bits),*];
                    if #bit + shamt < REG_BITS {
                        let src = src_bits[#bit - shamt];
                        src - #dst
                    } else {
                        #dst - #fill
                    }
                })
            }
        }
    }

    res
}
