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

#[derive(Clone, Debug)]
pub struct Constraint {
    pub expr: Expr,
    pub degree: u8,
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
