extern crate proc_macro;
use proc_macro::TokenStream;

use quote::quote;
use syn::{
    parse::{Parse, ParseStream},
    punctuated::Punctuated,
    *,
};

mod kw {
    syn::custom_keyword!(name);
    syn::custom_keyword!(opcode);
    syn::custom_keyword!(funct3);
    syn::custom_keyword!(parse);
    syn::custom_keyword!(constraints);
}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
enum Field {
    Imm,
    Uimm,
    Rd,
    Rs1,
    Rs2,
}

impl Parse for Field {
    fn parse(input: ParseStream) -> Result<Self> {
        let ident: Ident = input.parse()?;
        match ident.to_string().as_str() {
            "imm" => Ok(Self::Imm),
            "uimm" => Ok(Self::Uimm),
            "rd" => Ok(Self::Rd),
            "rs1" => Ok(Self::Rs1),
            "rs2" => Ok(Self::Rs2),
            other => Err(Error::new(ident.span(), format!("unknown field: {other}"))),
        }
    }
}

struct Constraint {
    expr: Expr,
    degree: u8,
}

impl Parse for Constraint {
    fn parse(input: ParseStream) -> Result<Self> {
        let expr: Expr = input.parse()?;
        input.parse::<Token![=>]>()?;
        let degree: Expr = input.parse()?;
        match degree {
            Expr::Lit(ExprLit {
                lit: Lit::Int(val), ..
            }) => {
                let value = val.base10_parse::<u8>().unwrap();
                Ok(Constraint {
                    expr,
                    degree: value,
                })
            }
            _ => unimplemented!("only integer literals are supported"),
        }
    }
}

struct Air {
    name: Ident,
    opcode: ExprArray,
    funct3: Option<ExprArray>,
    parse: Punctuated<Field, Token![/]>,
    constraints: Punctuated<Constraint, Token![,]>,
}

fn lit_to_array<const N: usize>(expr: Expr) -> syn::ExprArray {
    match expr {
        Expr::Lit(ExprLit {
            lit: Lit::Str(val), ..
        }) => {
            let mut expr_array = syn::ExprArray {
                attrs: vec![],
                bracket_token: Default::default(),
                elems: Punctuated::new(),
            };
            let value = val.value();
            assert_eq!(
                N,
                value.chars().count(),
                "literal length does not match expected length"
            );
            for c in value.chars() {
                assert!(c == '0' || c == '1', "only binary literals are supported");
                if c == '1' {
                    expr_array.elems.push(Expr::Path(ExprPath {
                        attrs: vec![],
                        qself: None,
                        path: syn::parse_str("E::ONE").unwrap(),
                    }));
                } else {
                    expr_array.elems.push(Expr::Path(ExprPath {
                        attrs: vec![],
                        qself: None,
                        path: syn::parse_str("E::ZERO").unwrap(),
                    }));
                }
            }
            expr_array
        }
        _ => unimplemented!("only str literals are supported"),
    }
}

impl Parse for Air {
    fn parse(input: ParseStream) -> Result<Self> {
        // constraint name
        input.parse::<kw::name>()?;
        input.parse::<Token![=]>()?;
        let name: Ident = input.parse()?;
        input.parse::<Token![,]>()?;

        // opcode
        input.parse::<kw::opcode>()?;
        input.parse::<Token![=]>()?;
        let opcode: Expr = input.parse()?;
        input.parse::<Token![,]>()?;

        // funct3 value, if any
        let mut funct3 = None;
        if input.peek(kw::funct3) {
            input.parse::<kw::funct3>()?;
            input.parse::<Token![=]>()?;
            let fn3: Expr = input.parse()?;
            funct3 = Some(fn3);
            input.parse::<Token![,]>()?;
        }

        // fields to read and make available to the constraint
        input.parse::<kw::parse>()?;
        input.parse::<Token![=]>()?;
        let parse = Punctuated::parse_separated_nonempty(input)?;
        input.parse::<Token![,]>()?;

        // constraints
        input.parse::<kw::constraints>()?;
        input.parse::<Token![=]>()?;

        let constraints = Punctuated::parse_separated_nonempty(input)?;

        Ok(Air {
            name,
            opcode: lit_to_array::<7>(opcode),
            funct3: funct3.map(lit_to_array::<3>),
            parse,
            constraints,
        })
    }
}

macro_rules! make_reg_flags {
    ($parse:expr, $field:expr) => {
        if $parse.iter().any(|f| *f == $field) {
            (quote! { binary_flag(&to_binary(reg, E::ZERO, E::ONE), test, E::ONE)}, 5)
        } else {
            (quote! { E::ONE }, 0)
        }
    };
}

#[proc_macro]
pub fn air(item: TokenStream) -> TokenStream {
    let Air {
        name,
        opcode,
        funct3,
        parse,
        constraints,
    } = parse_macro_input!(item);

    let funct3_deg = if funct3.is_some() { 3 } else { 0 };
    let funct3_flag_contents = if let Some(funct3) = funct3 {
        quote! {
            binary_flag(&#funct3, test, E::ONE)
        }
    } else {
        quote! {
            E::ONE
        }
    };

    let (rs1_flag_contents, rs1_deg) = make_reg_flags!(parse, Field::Rs1);
    let (rs2_flag_contents, rs2_deg) = make_reg_flags!(parse, Field::Rs2);
    let (rd_flag_contents, rd_deg) = make_reg_flags!(parse, Field::Rd);

    let c_exprs = constraints.iter().map(|c| &c.expr);
    let c_degs = constraints.iter().map(|c| c.degree);
    let n_constraints = constraints.len();

    quote! {
            pub mod #name {
                use trace_defs::*;
                use core::ops::*;
                use winterfell::{EvaluationFrame, TransitionConstraintDegree, math::FieldElement};

                const OPCODE_FLAG_DEG: usize = 7;
                const FUNCT3_FLAG_DEG: usize = #funct3_deg as usize;
                const RS1_FLAG_DEG: usize = #rs1_deg as usize;
                const RS2_FLAG_DEG: usize = #rs2_deg as usize;
                const RD_FLAG_DEG: usize = #rd_deg as usize;
                const RD_CNT: usize = 1 << #rd_deg as usize;
                const RS1_CNT: usize = 1 << #rs1_deg as usize;
                const RS2_CNT: usize = 1 << #rs2_deg as usize;
                const TOT_CNT: usize = RD_CNT * RS1_CNT * RS2_CNT;
                const BODY_FLAG_DEG: usize = 1;
                const CONSTRAINT_DEGS: [usize; #n_constraints] = [#(#c_degs as usize),*];

                pub fn evaluate_transitions<E: FieldElement>(
                    frame: &EvaluationFrame<E>,
                    periodic_values: &[E],
                    result: &mut [E],
                ) -> usize {
                    let current = frame.current();
                    let next = frame.next();
                    let is_body = current[BODY];
                    let mut index = 0;

                    for rd in 0..RD_CNT {
                        for rs1 in 0..RS1_CNT {
                            for rs2 in 0..RS2_CNT {
                               
                                let rd_flag = rd_flag(rd as u8, &current[RD_END..RD_END + 5]);
                                let rs1_flag = rs1_flag(rs1 as u8, &current[RS1_END..RS1_END + 5]);
                                let rs2_flag = rs2_flag(rs2 as u8, &current[RS2_END..RS2_END + 5]);
                                let funct3_flag = funct3_flag(&current[FUNCT3_END..FUNCT3_END + 3]);
                                let op_flag = op_flag(&current[OPCODE_END..OPCODE_END + 7]);
                                let body_flag = current[BODY];

                                let cumulative_flag = op_flag * rd_flag * rs1_flag * rs2_flag * body_flag * funct3_flag;
                                let imm = get_immediate(&current[UIMM_END..UIMM_END + 12]);
                                let uimm = get_immediate(&current[UIMM_END..UIMM_END + 20]);
                                let pc = current[PC];
                                let h0 = current[H_0];
                                let h1 = current[H_1];
                                let h2 = current[H_2];
                                let h3 = current[H_3];
                                let h4 = current[H_4];
                                let h5 = current[H_5];
                                let rd = next[REGISTER_START + rd];
                                let rs1 = current[REGISTER_START + rs1];
                                let rs2 = current[REGISTER_START + rs2];
                                
                                #(
                                    result[index] = (#c_exprs) * cumulative_flag;
                                    index += 1;
                                )*
                            }
                        }
                    }

                    // return the number of use constraint columns
                    #n_constraints
                }

                pub fn constraint_degrees() -> Vec<TransitionConstraintDegree> {
                    let mut degrees = Vec::with_capacity(TOT_CNT);
                    for _ in 0..TOT_CNT {
                        for deg in CONSTRAINT_DEGS.iter() {
                            degrees.push(TransitionConstraintDegree::new( OPCODE_FLAG_DEG + RD_FLAG_DEG + RS1_FLAG_DEG + RS2_FLAG_DEG + FUNCT3_FLAG_DEG + BODY_FLAG_DEG + deg));
                        }
                    }
                    degrees
                }

                fn evaluate_constraints<E: FieldElement>(current: &[E], next: &[E], rd: usize, rs1: usize, rs2: usize, result: &mut [E]) {

                }

                fn rd_flag<E: FieldElement>(reg: u8, test: &[E]) -> E {
                    assert_eq!(test.len(), 5);
                    #rd_flag_contents
                }

                fn rs1_flag<E: FieldElement>(reg: u8, test: &[E]) -> E {
                    assert_eq!(test.len(), 5);
                    #rs1_flag_contents
                }

                fn rs2_flag<E: FieldElement>(reg: u8, test: &[E]) -> E {
                    assert_eq!(test.len(), 5);
                    #rs2_flag_contents
                }

                fn funct3_flag<E>(test: &[E]) -> E
                where
                    E: FieldElement,
                {
                    assert_eq!(test.len(), 3);
                    #funct3_flag_contents
                }

                // Degree: 6
                fn op_flag<E>(test: &[E]) -> E
                where
                    E: FieldElement,
                {
                    assert_eq!(test.len(), 7);
                    binary_flag(&#opcode, test, E::ONE)
                }

                // Degree: 4
                fn reg_flag<E>(reg: u8, test: &[E]) -> E
                where
                    E: FieldElement,
                {
                    assert_eq!(test.len(), 5);
                    binary_flag(&to_binary(reg, E::ZERO, E::ONE), test, E::ONE)
                }

                pub fn binary_flag<E>(expected: &[E], test: &[E], one: E) -> E
                where
                    E: Mul<Output = E> + Sub<Output = E> + Copy + FieldElement,
                {
                    let mut result = one;
                    for (i, bit) in expected.iter().enumerate() {
                        result *= if bit == &one { test[i] } else { one - test[i] };
                    }
                    result
                }

                fn to_binary<E: Copy>(reg: u8, zero: E, one: E) -> [E; 5] {
                    let mut result = [zero; 5];
                    for i in 5..0 {
                        if reg & (1 << i) != 0 {
                            result[i] = one;
                        }
                    }

                    result
                }

                // Degree: 1
                // TODO: this is unsigned, but we need signed
                fn get_immediate<E: FieldElement>(op: &[E]) -> E {
                    let mut result = E::ZERO;
                    assert_eq!(op.len(), 12);
                    for (i, bit) in op.iter().enumerate() {
                        result += *bit * E::from(1u32 << i);
                    }
                    result
                }
        }
    }.into()
}
