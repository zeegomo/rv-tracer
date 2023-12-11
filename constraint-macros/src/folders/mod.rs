use syn::{fold::Fold, Expr, ExprPath};

/// Extesions to support nice syntax for constraints.
///
// Replace a constraint of the form rs1 * rs2 = rd with a constraint operating on a single bit
// of rs1, rs2 and rd
// rsx = rsx[..]
// Additionally support
struct BitwiseFold {
    bit: usize,
}

impl Fold for BitwiseFold {
    // The only allowed operations are addition, subtraction and multiplication, which are all binary operations
    fn fold_expr(&mut self, mut i: Expr) -> Expr {
        i = replace_if_reg(i, "rs1", self.bit, "current");
        i = replace_if_reg(i, "rs2", self.bit, "current");
        i = replace_if_reg(i, "rd", self.bit, "next");
        i
    }
}

fn replace_if_reg(original: Expr, reg: &str, bit: usize, base: &str) -> Expr {
    let offset = format!("{}_START", reg.to_uppercase());
    let reg = Expr::Path(ExprPath {
        attrs: vec![],
        qself: None,
        path: syn::parse_str(reg).unwrap(),
    });
    if original == reg {
        syn::parse_str(&format!("{base}[{offset}+{bit}]")).unwrap()
    } else {
        original
    }
}

pub fn fold_bitwise(expr: Expr, n_bits: usize) -> Vec<Expr> {
    let mut res = Vec::with_capacity(n_bits);
    for i in 0..n_bits {
        let mut fold = BitwiseFold { bit: i };
        res.push(fold.fold_expr(expr.clone()));
    }
    res
}

// we might want to use
