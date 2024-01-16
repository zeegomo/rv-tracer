mod folders;
use crate::REG_BITS;
use proc_macro2::TokenStream;
use quote::ToTokens;
use syn::{
    parse::{Parse, ParseStream},
    punctuated::Punctuated,
    *,
};

pub mod kw {
    syn::custom_keyword!(name);
    syn::custom_keyword!(opcode);
    syn::custom_keyword!(funct3);
    syn::custom_keyword!(parse);
    syn::custom_keyword!(constraints);
    syn::custom_keyword!(bitwise);
    syn::custom_keyword!(sticky);
    syn::custom_keyword!(shift);
}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum Field {
    Imm,
    Uimm,
    Rd,
    Rs1,
    Rs2,
    Shamt,
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
            "shamt" => Ok(Self::Shamt),
            other => Err(Error::new(ident.span(), format!("unknown field: {other}"))),
        }
    }
}

impl Field {
    pub fn to_trace_bit(self, bit: usize) -> Expr {
        assert!(bit < REG_BITS, "bit index out of bounds");
        let bit = REG_BITS - bit - 1;
        match self {
            Field::Imm => todo!(),
            Field::Uimm => todo!(),
            Field::Rd => syn::parse_str(&format!("next[RS1_BITS_END + {}]", bit)).unwrap(),
            Field::Rs1 => syn::parse_str(&format!("current[RS1_BITS_END + {}]", bit)).unwrap(),
            Field::Rs2 => syn::parse_str(&format!("current[RS2_BITS_END + {}]", bit)).unwrap(),
            Field::Shamt => todo!(),
        }
    }
}

#[derive(Clone, Debug)]
pub enum Constraint {
    Plain(PlainConstraint),
    Bitwise(PlainConstraint),
    Shift(ShiftConstraint),
}

#[derive(Clone, Debug)]
pub struct PlainConstraint {
    pub expr: Expr,
    pub degree: u8,
}

impl Parse for Constraint {
    fn parse(input: ParseStream) -> Result<Self> {
        if input.peek(kw::shift) {
            input.parse::<kw::shift>()?;
            Ok(Constraint::Shift(input.parse()?))
        } else if input.peek(kw::bitwise) {
            input.parse::<kw::bitwise>()?;
            Ok(Constraint::Bitwise(input.parse()?))
        } else {
            Ok(Constraint::Plain(input.parse()?))
        }
    }
}

impl Constraint {
    pub fn degree<const REG_BITS: usize>(&self) -> Vec<u8> {
        match self {
            Constraint::Plain(c) => vec![c.degree],
            Constraint::Bitwise(c) => vec![c.degree; REG_BITS],
            Constraint::Shift(_) => vec![1; REG_BITS],
        }
    }

    // This will return the expression that should be evaluated to determine if the constraint is satisfied.
    // It will return:
    // * a single expression for plain constraints
    // * a vector of reg_bits expressions for bitwise constraints and shift constraints.
    pub fn into_token_stream<const REG_BITS: usize>(self) -> Vec<TokenStream> {
        match self {
            Constraint::Plain(c) => vec![c.expr.to_token_stream()],
            Constraint::Bitwise(c) => folders::fold_bitwise::<REG_BITS>(c.expr)
                .into_iter()
                .map(|x| x.to_token_stream())
                .collect(),
            Constraint::Shift(c) => folders::generate_shift_constraints::<REG_BITS>(c),
        }
    }
}

impl Parse for PlainConstraint {
    fn parse(input: ParseStream) -> Result<Self> {
        let expr: Expr = input.parse()?;
        input.parse::<Token![=>]>()?;
        let degree = lit_expr_to_u8(input.parse()?);
        Ok(Self { expr, degree })
    }
}

fn lit_expr_to_u8(expr: Expr) -> u8 {
    match expr {
        Expr::Lit(ExprLit {
            lit: Lit::Int(val), ..
        }) => val.base10_parse::<u8>().unwrap(),
        _ => unimplemented!("only integer literals are supported"),
    }
}

pub struct Air {
    pub name: Ident,
    pub opcode: ExprArray,
    pub funct3: Option<ExprArray>,
    pub parse: Punctuated<Field, Token![/]>,
    pub constraints: Punctuated<Constraint, Token![,]>,
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

#[derive(Clone, Debug, Copy)]
pub enum Kind {
    Left,
    Right,
    RightSticky,
}

#[derive(Clone, Debug)]
pub struct ShiftConstraint {
    pub src: Field,
    pub dst: Field,
    pub kind: Kind,
}

impl Parse for ShiftConstraint {
    fn parse(input: ParseStream) -> Result<Self> {
        let f1 = input.parse()?;
        if input.peek(Token![-]) {
            // right shift
            input.parse::<Token![-]>()?;
            input.parse::<Token![>]>()?;

            let sticky = input.peek(kw::sticky);
            if sticky {
                input.parse::<kw::sticky>().unwrap();
                input.parse::<Token![-]>()?;
                input.parse::<Token![>]>()?;
            }

            let f2 = input.parse()?;

            Ok(Self {
                src: f1,
                dst: f2,
                kind: if sticky {
                    Kind::RightSticky
                } else {
                    Kind::Right
                },
            })
        } else {
            // left shift
            input.parse::<Token![<]>()?;
            input.parse::<Token![-]>()?;

            let f2 = input.parse()?;
            Ok(Self {
                src: f2,
                dst: f1,
                kind: Kind::Left,
            })
        }
    }
}
