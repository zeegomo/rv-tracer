#![feature(prelude_import)]
#[prelude_import]
use std::prelude::rust_2021::*;
#[macro_use]
extern crate std;
extern crate proc_macro;
use proc_macro::TokenStream;
use proc_macro2::Span;
use quote::quote;
use syn::{
    parse::{Parse, ParseStream},
    punctuated::Punctuated,
    *,
};
use winterfell::math::FieldElement;
const N_REGISTERS: usize = 32;
mod kw {
    #[allow(non_camel_case_types)]
    pub struct name {
        pub span: ::syn::__private::Span,
    }
    #[doc(hidden)]
    #[allow(dead_code, non_snake_case)]
    pub fn name<__S: ::syn::__private::IntoSpans<::syn::__private::Span>>(span: __S) -> name {
        name {
            span: ::syn::__private::IntoSpans::into_spans(span),
        }
    }
    const _: () = {
        impl ::syn::__private::Default for name {
            fn default() -> Self {
                name {
                    span: ::syn::__private::Span::call_site(),
                }
            }
        }
        impl ::syn::__private::CustomToken for name {
            fn peek(cursor: ::syn::buffer::Cursor) -> ::syn::__private::bool {
                if let ::syn::__private::Some((ident, _rest)) = cursor.ident() {
                    ident == "name"
                } else {
                    false
                }
            }
            fn display() -> &'static ::syn::__private::str {
                "`name`"
            }
        }
        impl ::syn::parse::Parse for name {
            fn parse(input: ::syn::parse::ParseStream) -> ::syn::parse::Result<name> {
                input.step(|cursor| {
                    if let ::syn::__private::Some((ident, rest)) = cursor.ident() {
                        if ident == "name" {
                            return ::syn::__private::Ok((name { span: ident.span() }, rest));
                        }
                    }
                    ::syn::__private::Err(cursor.error("expected `name`"))
                })
            }
        }
        impl ::syn::__private::ToTokens for name {
            fn to_tokens(&self, tokens: &mut ::syn::__private::TokenStream2) {
                let ident = ::syn::Ident::new("name", self.span);
                ::syn::__private::TokenStreamExt::append(tokens, ident);
            }
        }
        impl ::syn::__private::Copy for name {}
        #[allow(clippy::expl_impl_clone_on_copy)]
        impl ::syn::__private::Clone for name {
            fn clone(&self) -> Self {
                *self
            }
        }
    };
    #[allow(non_camel_case_types)]
    pub struct opcode {
        pub span: ::syn::__private::Span,
    }
    #[doc(hidden)]
    #[allow(dead_code, non_snake_case)]
    pub fn opcode<__S: ::syn::__private::IntoSpans<::syn::__private::Span>>(span: __S) -> opcode {
        opcode {
            span: ::syn::__private::IntoSpans::into_spans(span),
        }
    }
    const _: () = {
        impl ::syn::__private::Default for opcode {
            fn default() -> Self {
                opcode {
                    span: ::syn::__private::Span::call_site(),
                }
            }
        }
        impl ::syn::__private::CustomToken for opcode {
            fn peek(cursor: ::syn::buffer::Cursor) -> ::syn::__private::bool {
                if let ::syn::__private::Some((ident, _rest)) = cursor.ident() {
                    ident == "opcode"
                } else {
                    false
                }
            }
            fn display() -> &'static ::syn::__private::str {
                "`opcode`"
            }
        }
        impl ::syn::parse::Parse for opcode {
            fn parse(input: ::syn::parse::ParseStream) -> ::syn::parse::Result<opcode> {
                input.step(|cursor| {
                    if let ::syn::__private::Some((ident, rest)) = cursor.ident() {
                        if ident == "opcode" {
                            return ::syn::__private::Ok((opcode { span: ident.span() }, rest));
                        }
                    }
                    ::syn::__private::Err(cursor.error("expected `opcode`"))
                })
            }
        }
        impl ::syn::__private::ToTokens for opcode {
            fn to_tokens(&self, tokens: &mut ::syn::__private::TokenStream2) {
                let ident = ::syn::Ident::new("opcode", self.span);
                ::syn::__private::TokenStreamExt::append(tokens, ident);
            }
        }
        impl ::syn::__private::Copy for opcode {}
        #[allow(clippy::expl_impl_clone_on_copy)]
        impl ::syn::__private::Clone for opcode {
            fn clone(&self) -> Self {
                *self
            }
        }
    };
    #[allow(non_camel_case_types)]
    pub struct funct3 {
        pub span: ::syn::__private::Span,
    }
    #[doc(hidden)]
    #[allow(dead_code, non_snake_case)]
    pub fn funct3<__S: ::syn::__private::IntoSpans<::syn::__private::Span>>(span: __S) -> funct3 {
        funct3 {
            span: ::syn::__private::IntoSpans::into_spans(span),
        }
    }
    const _: () = {
        impl ::syn::__private::Default for funct3 {
            fn default() -> Self {
                funct3 {
                    span: ::syn::__private::Span::call_site(),
                }
            }
        }
        impl ::syn::__private::CustomToken for funct3 {
            fn peek(cursor: ::syn::buffer::Cursor) -> ::syn::__private::bool {
                if let ::syn::__private::Some((ident, _rest)) = cursor.ident() {
                    ident == "funct3"
                } else {
                    false
                }
            }
            fn display() -> &'static ::syn::__private::str {
                "`funct3`"
            }
        }
        impl ::syn::parse::Parse for funct3 {
            fn parse(input: ::syn::parse::ParseStream) -> ::syn::parse::Result<funct3> {
                input.step(|cursor| {
                    if let ::syn::__private::Some((ident, rest)) = cursor.ident() {
                        if ident == "funct3" {
                            return ::syn::__private::Ok((funct3 { span: ident.span() }, rest));
                        }
                    }
                    ::syn::__private::Err(cursor.error("expected `funct3`"))
                })
            }
        }
        impl ::syn::__private::ToTokens for funct3 {
            fn to_tokens(&self, tokens: &mut ::syn::__private::TokenStream2) {
                let ident = ::syn::Ident::new("funct3", self.span);
                ::syn::__private::TokenStreamExt::append(tokens, ident);
            }
        }
        impl ::syn::__private::Copy for funct3 {}
        #[allow(clippy::expl_impl_clone_on_copy)]
        impl ::syn::__private::Clone for funct3 {
            fn clone(&self) -> Self {
                *self
            }
        }
    };
    #[allow(non_camel_case_types)]
    pub struct parse {
        pub span: ::syn::__private::Span,
    }
    #[doc(hidden)]
    #[allow(dead_code, non_snake_case)]
    pub fn parse<__S: ::syn::__private::IntoSpans<::syn::__private::Span>>(span: __S) -> parse {
        parse {
            span: ::syn::__private::IntoSpans::into_spans(span),
        }
    }
    const _: () = {
        impl ::syn::__private::Default for parse {
            fn default() -> Self {
                parse {
                    span: ::syn::__private::Span::call_site(),
                }
            }
        }
        impl ::syn::__private::CustomToken for parse {
            fn peek(cursor: ::syn::buffer::Cursor) -> ::syn::__private::bool {
                if let ::syn::__private::Some((ident, _rest)) = cursor.ident() {
                    ident == "parse"
                } else {
                    false
                }
            }
            fn display() -> &'static ::syn::__private::str {
                "`parse`"
            }
        }
        impl ::syn::parse::Parse for parse {
            fn parse(input: ::syn::parse::ParseStream) -> ::syn::parse::Result<parse> {
                input.step(|cursor| {
                    if let ::syn::__private::Some((ident, rest)) = cursor.ident() {
                        if ident == "parse" {
                            return ::syn::__private::Ok((parse { span: ident.span() }, rest));
                        }
                    }
                    ::syn::__private::Err(cursor.error("expected `parse`"))
                })
            }
        }
        impl ::syn::__private::ToTokens for parse {
            fn to_tokens(&self, tokens: &mut ::syn::__private::TokenStream2) {
                let ident = ::syn::Ident::new("parse", self.span);
                ::syn::__private::TokenStreamExt::append(tokens, ident);
            }
        }
        impl ::syn::__private::Copy for parse {}
        #[allow(clippy::expl_impl_clone_on_copy)]
        impl ::syn::__private::Clone for parse {
            fn clone(&self) -> Self {
                *self
            }
        }
    };
    #[allow(non_camel_case_types)]
    pub struct constraints {
        pub span: ::syn::__private::Span,
    }
    #[doc(hidden)]
    #[allow(dead_code, non_snake_case)]
    pub fn constraints<__S: ::syn::__private::IntoSpans<::syn::__private::Span>>(
        span: __S,
    ) -> constraints {
        constraints {
            span: ::syn::__private::IntoSpans::into_spans(span),
        }
    }
    const _: () = {
        impl ::syn::__private::Default for constraints {
            fn default() -> Self {
                constraints {
                    span: ::syn::__private::Span::call_site(),
                }
            }
        }
        impl ::syn::__private::CustomToken for constraints {
            fn peek(cursor: ::syn::buffer::Cursor) -> ::syn::__private::bool {
                if let ::syn::__private::Some((ident, _rest)) = cursor.ident() {
                    ident == "constraints"
                } else {
                    false
                }
            }
            fn display() -> &'static ::syn::__private::str {
                "`constraints`"
            }
        }
        impl ::syn::parse::Parse for constraints {
            fn parse(input: ::syn::parse::ParseStream) -> ::syn::parse::Result<constraints> {
                input.step(|cursor| {
                    if let ::syn::__private::Some((ident, rest)) = cursor.ident() {
                        if ident == "constraints" {
                            return ::syn::__private::Ok((
                                constraints { span: ident.span() },
                                rest,
                            ));
                        }
                    }
                    ::syn::__private::Err(cursor.error("expected `constraints`"))
                })
            }
        }
        impl ::syn::__private::ToTokens for constraints {
            fn to_tokens(&self, tokens: &mut ::syn::__private::TokenStream2) {
                let ident = ::syn::Ident::new("constraints", self.span);
                ::syn::__private::TokenStreamExt::append(tokens, ident);
            }
        }
        impl ::syn::__private::Copy for constraints {}
        #[allow(clippy::expl_impl_clone_on_copy)]
        impl ::syn::__private::Clone for constraints {
            fn clone(&self) -> Self {
                *self
            }
        }
    };
}
enum Field {
    Imm,
    Uimm,
    Rd,
    Rs1,
    Rs2,
}
#[automatically_derived]
impl ::core::fmt::Debug for Field {
    fn fmt(&self, f: &mut ::core::fmt::Formatter) -> ::core::fmt::Result {
        ::core::fmt::Formatter::write_str(
            f,
            match self {
                Field::Imm => "Imm",
                Field::Uimm => "Uimm",
                Field::Rd => "Rd",
                Field::Rs1 => "Rs1",
                Field::Rs2 => "Rs2",
            },
        )
    }
}
#[automatically_derived]
impl ::core::marker::Copy for Field {}
#[automatically_derived]
impl ::core::clone::Clone for Field {
    #[inline]
    fn clone(&self) -> Field {
        *self
    }
}
#[automatically_derived]
impl ::core::marker::StructuralPartialEq for Field {}
#[automatically_derived]
impl ::core::cmp::PartialEq for Field {
    #[inline]
    fn eq(&self, other: &Field) -> bool {
        let __self_tag = ::core::intrinsics::discriminant_value(self);
        let __arg1_tag = ::core::intrinsics::discriminant_value(other);
        __self_tag == __arg1_tag
    }
}
#[automatically_derived]
impl ::core::marker::StructuralEq for Field {}
#[automatically_derived]
impl ::core::cmp::Eq for Field {
    #[inline]
    #[doc(hidden)]
    #[no_coverage]
    fn assert_receiver_is_total_eq(&self) -> () {}
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
            other => Err(Error::new(ident.span(), {
                let res = ::alloc::fmt::format(format_args!("unknown field: {0}", other));
                res
            })),
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
        input.parse::<::syn::token::FatArrow>()?;
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
            _ => {
                ::core::panicking::panic_fmt(format_args!(
                    "not implemented: {0}",
                    format_args!("only integer literals are supported"),
                ));
            }
        }
    }
}
struct Air {
    name: Ident,
    opcode: ExprArray,
    funct3: Option<ExprArray>,
    parse: Punctuated<Field, ::syn::token::Slash>,
    constraints: Punctuated<Constraint, ::syn::token::Comma>,
}
fn lit_to_array<const N: usize>(expr: Expr) -> syn::ExprArray {
    match expr {
        Expr::Lit(ExprLit {
            lit: Lit::Str(val), ..
        }) => {
            let mut expr_array = syn::ExprArray {
                attrs: ::alloc::vec::Vec::new(),
                bracket_token: Default::default(),
                elems: Punctuated::new(),
            };
            let value = val.value();
            match (&N, &value.chars().count()) {
                (left_val, right_val) => {
                    if !(*left_val == *right_val) {
                        let kind = ::core::panicking::AssertKind::Eq;
                        ::core::panicking::assert_failed(
                            kind,
                            &*left_val,
                            &*right_val,
                            ::core::option::Option::Some(format_args!(
                                "literal length does not match expected length",
                            )),
                        );
                    }
                }
            };
            for c in value.chars() {
                if !(c == '0' || c == '1') {
                    {
                        ::core::panicking::panic_fmt(format_args!(
                            "only binary literals are supported"
                        ));
                    }
                }
                if c == '1' {
                    expr_array.elems.push(Expr::Path(ExprPath {
                        attrs: ::alloc::vec::Vec::new(),
                        qself: None,
                        path: syn::parse_str("E::ONE").unwrap(),
                    }));
                } else {
                    expr_array.elems.push(Expr::Path(ExprPath {
                        attrs: ::alloc::vec::Vec::new(),
                        qself: None,
                        path: syn::parse_str("E::ZERO").unwrap(),
                    }));
                }
            }
            expr_array
        }
        _ => {
            ::core::panicking::panic_fmt(format_args!(
                "not implemented: {0}",
                format_args!("only str literals are supported"),
            ));
        }
    }
}
impl Parse for Air {
    fn parse(input: ParseStream) -> Result<Self> {
        input.parse::<kw::name>()?;
        input.parse::<::syn::token::Eq>()?;
        let name: Ident = input.parse()?;
        input.parse::<::syn::token::Comma>()?;
        input.parse::<kw::opcode>()?;
        input.parse::<::syn::token::Eq>()?;
        let opcode: Expr = input.parse()?;
        input.parse::<::syn::token::Comma>()?;
        let mut funct3 = None;
        if input.peek(kw::funct3) {
            input.parse::<kw::funct3>()?;
            input.parse::<::syn::token::Eq>()?;
            let fn3: Expr = input.parse()?;
            funct3 = Some(fn3);
        }
        input.parse::<::syn::token::Comma>()?;
        input.parse::<kw::parse>()?;
        input.parse::<::syn::token::Eq>()?;
        let parse = Punctuated::parse_separated_nonempty(input)?;
        input.parse::<::syn::token::Comma>()?;
        input.parse::<kw::constraints>()?;
        input.parse::<::syn::token::Eq>()?;
        let constraints = Punctuated::parse_separated_nonempty(input)?;
        Ok(Air {
            name,
            opcode: lit_to_array::<7>(opcode),
            funct3: funct3.map(|f| lit_to_array::<3>(f)),
            parse,
            constraints,
        })
    }
}
#[proc_macro]
pub fn air(item: TokenStream) -> TokenStream {
    let Air {
        name,
        opcode,
        funct3,
        parse,
        constraints,
    } = match ::syn::parse::<_>(item) {
        ::syn::__private::Ok(data) => data,
        ::syn::__private::Err(err) => {
            return ::syn::__private::TokenStream::from(err.to_compile_error());
        }
    };
    let funct3_deg = if funct3.is_some() { 3 } else { 0 };
    let rs1_deg = if parse.iter().any(|f| *f == Field::Rs1) {
        5
    } else {
        0
    };
    let rs2_deg = if parse.iter().any(|f| *f == Field::Rs2) {
        5
    } else {
        0
    };
    let rd_deg = if parse.iter().any(|f| *f == Field::Rd) {
        5
    } else {
        0
    };
    let c_exprs = constraints.iter().map(|c| &c.expr);
    let c_degs = constraints.iter().map(|c| c.degree);
    let funct3_flag_contents = if let Some(funct3) = funct3 {
        {
            let mut _s = ::quote::__private::TokenStream::new();
            ::quote::__private::push_ident(&mut _s, "binary_flag");
            ::quote::__private::push_group(&mut _s, ::quote::__private::Delimiter::Parenthesis, {
                let mut _s = ::quote::__private::TokenStream::new();
                ::quote::__private::push_and(&mut _s);
                ::quote::ToTokens::to_tokens(&funct3, &mut _s);
                ::quote::__private::push_comma(&mut _s);
                ::quote::__private::push_ident(&mut _s, "test");
                ::quote::__private::push_comma(&mut _s);
                ::quote::__private::push_ident(&mut _s, "E");
                ::quote::__private::push_colon2(&mut _s);
                ::quote::__private::push_ident(&mut _s, "ONE");
                _s
            });
            _s
        }
    } else {
        {
            let mut _s = ::quote::__private::TokenStream::new();
            ::quote::__private::push_ident(&mut _s, "E");
            ::quote::__private::push_colon2(&mut _s);
            ::quote::__private::push_ident(&mut _s, "ONE");
            _s
        }
    };
    let n_constraints = constraints.len();
    {
        let mut _s = ::quote::__private::TokenStream::new();
        ::quote::__private::push_ident(&mut _s, "pub");
        ::quote::__private::push_ident(&mut _s, "mod");
        ::quote::ToTokens::to_tokens(&name, &mut _s);
        ::quote::__private::push_group(&mut _s, ::quote::__private::Delimiter::Brace, {
            let mut _s = ::quote::__private::TokenStream::new();
            ::quote::__private::push_ident(&mut _s, "use");
            ::quote::__private::push_ident(&mut _s, "trace_defs");
            ::quote::__private::push_colon2(&mut _s);
            ::quote::__private::push_star(&mut _s);
            ::quote::__private::push_semi(&mut _s);
            ::quote::__private::push_ident(&mut _s, "use");
            ::quote::__private::push_ident(&mut _s, "core");
            ::quote::__private::push_colon2(&mut _s);
            ::quote::__private::push_ident(&mut _s, "ops");
            ::quote::__private::push_colon2(&mut _s);
            ::quote::__private::push_star(&mut _s);
            ::quote::__private::push_semi(&mut _s);
            ::quote::__private::push_ident(&mut _s, "use");
            ::quote::__private::push_ident(&mut _s, "winterfell");
            ::quote::__private::push_colon2(&mut _s);
            ::quote::__private::push_group(&mut _s, ::quote::__private::Delimiter::Brace, {
                let mut _s = ::quote::__private::TokenStream::new();
                ::quote::__private::push_ident(&mut _s, "EvaluationFrame");
                ::quote::__private::push_comma(&mut _s);
                ::quote::__private::push_ident(&mut _s, "TransitionConstraintDegree");
                ::quote::__private::push_comma(&mut _s);
                ::quote::__private::push_ident(&mut _s, "math");
                ::quote::__private::push_colon2(&mut _s);
                ::quote::__private::push_ident(&mut _s, "FieldElement");
                _s
            });
            ::quote::__private::push_semi(&mut _s);
            ::quote::__private::push_ident(&mut _s, "const");
            ::quote::__private::push_ident(&mut _s, "OPCODE_FLAG_DEG");
            ::quote::__private::push_colon(&mut _s);
            ::quote::__private::push_ident(&mut _s, "usize");
            ::quote::__private::push_eq(&mut _s);
            ::quote::__private::parse(&mut _s, "7");
            ::quote::__private::push_semi(&mut _s);
            ::quote::__private::push_ident(&mut _s, "const");
            ::quote::__private::push_ident(&mut _s, "FUNCT3_FLAG_DEG");
            ::quote::__private::push_colon(&mut _s);
            ::quote::__private::push_ident(&mut _s, "usize");
            ::quote::__private::push_eq(&mut _s);
            ::quote::ToTokens::to_tokens(&funct3_deg, &mut _s);
            ::quote::__private::push_ident(&mut _s, "as");
            ::quote::__private::push_ident(&mut _s, "usize");
            ::quote::__private::push_semi(&mut _s);
            ::quote::__private::push_ident(&mut _s, "const");
            ::quote::__private::push_ident(&mut _s, "RS1_FLAG_DEG");
            ::quote::__private::push_colon(&mut _s);
            ::quote::__private::push_ident(&mut _s, "usize");
            ::quote::__private::push_eq(&mut _s);
            ::quote::ToTokens::to_tokens(&rs1_deg, &mut _s);
            ::quote::__private::push_ident(&mut _s, "as");
            ::quote::__private::push_ident(&mut _s, "usize");
            ::quote::__private::push_semi(&mut _s);
            ::quote::__private::push_ident(&mut _s, "const");
            ::quote::__private::push_ident(&mut _s, "RS2_FLAG_DEG");
            ::quote::__private::push_colon(&mut _s);
            ::quote::__private::push_ident(&mut _s, "usize");
            ::quote::__private::push_eq(&mut _s);
            ::quote::ToTokens::to_tokens(&rs2_deg, &mut _s);
            ::quote::__private::push_ident(&mut _s, "as");
            ::quote::__private::push_ident(&mut _s, "usize");
            ::quote::__private::push_semi(&mut _s);
            ::quote::__private::push_ident(&mut _s, "const");
            ::quote::__private::push_ident(&mut _s, "RD_FLAG_DEG");
            ::quote::__private::push_colon(&mut _s);
            ::quote::__private::push_ident(&mut _s, "usize");
            ::quote::__private::push_eq(&mut _s);
            ::quote::ToTokens::to_tokens(&rd_deg, &mut _s);
            ::quote::__private::push_ident(&mut _s, "as");
            ::quote::__private::push_ident(&mut _s, "usize");
            ::quote::__private::push_semi(&mut _s);
            ::quote::__private::push_ident(&mut _s, "const");
            ::quote::__private::push_ident(&mut _s, "RD_CNT");
            ::quote::__private::push_colon(&mut _s);
            ::quote::__private::push_ident(&mut _s, "usize");
            ::quote::__private::push_eq(&mut _s);
            ::quote::__private::parse(&mut _s, "1");
            ::quote::__private::push_shl(&mut _s);
            ::quote::ToTokens::to_tokens(&rd_deg, &mut _s);
            ::quote::__private::push_ident(&mut _s, "as");
            ::quote::__private::push_ident(&mut _s, "usize");
            ::quote::__private::push_semi(&mut _s);
            ::quote::__private::push_ident(&mut _s, "const");
            ::quote::__private::push_ident(&mut _s, "RS1_CNT");
            ::quote::__private::push_colon(&mut _s);
            ::quote::__private::push_ident(&mut _s, "usize");
            ::quote::__private::push_eq(&mut _s);
            ::quote::__private::parse(&mut _s, "1");
            ::quote::__private::push_shl(&mut _s);
            ::quote::ToTokens::to_tokens(&rs1_deg, &mut _s);
            ::quote::__private::push_ident(&mut _s, "as");
            ::quote::__private::push_ident(&mut _s, "usize");
            ::quote::__private::push_semi(&mut _s);
            ::quote::__private::push_ident(&mut _s, "const");
            ::quote::__private::push_ident(&mut _s, "RS2_CNT");
            ::quote::__private::push_colon(&mut _s);
            ::quote::__private::push_ident(&mut _s, "usize");
            ::quote::__private::push_eq(&mut _s);
            ::quote::__private::parse(&mut _s, "1");
            ::quote::__private::push_shl(&mut _s);
            ::quote::ToTokens::to_tokens(&rs2_deg, &mut _s);
            ::quote::__private::push_ident(&mut _s, "as");
            ::quote::__private::push_ident(&mut _s, "usize");
            ::quote::__private::push_semi(&mut _s);
            ::quote::__private::push_ident(&mut _s, "const");
            ::quote::__private::push_ident(&mut _s, "TOT_CNT");
            ::quote::__private::push_colon(&mut _s);
            ::quote::__private::push_ident(&mut _s, "usize");
            ::quote::__private::push_eq(&mut _s);
            ::quote::__private::push_ident(&mut _s, "RD_CNT");
            ::quote::__private::push_star(&mut _s);
            ::quote::__private::push_ident(&mut _s, "RS1_CNT");
            ::quote::__private::push_star(&mut _s);
            ::quote::__private::push_ident(&mut _s, "RS2_CNT");
            ::quote::__private::push_semi(&mut _s);
            ::quote::__private::push_ident(&mut _s, "const");
            ::quote::__private::push_ident(&mut _s, "N_REGISTERS");
            ::quote::__private::push_colon(&mut _s);
            ::quote::__private::push_ident(&mut _s, "usize");
            ::quote::__private::push_eq(&mut _s);
            ::quote::__private::parse(&mut _s, "32");
            ::quote::__private::push_semi(&mut _s);
            ::quote::__private::push_ident(&mut _s, "const");
            ::quote::__private::push_ident(&mut _s, "BODY_FLAG_DEG");
            ::quote::__private::push_colon(&mut _s);
            ::quote::__private::push_ident(&mut _s, "usize");
            ::quote::__private::push_eq(&mut _s);
            ::quote::__private::parse(&mut _s, "1");
            ::quote::__private::push_semi(&mut _s);
            ::quote::__private::push_ident(&mut _s, "const");
            ::quote::__private::push_ident(&mut _s, "CONSTRAINT_DEGS");
            ::quote::__private::push_colon(&mut _s);
            ::quote::__private::push_group(&mut _s, ::quote::__private::Delimiter::Bracket, {
                let mut _s = ::quote::__private::TokenStream::new();
                ::quote::__private::push_ident(&mut _s, "usize");
                ::quote::__private::push_semi(&mut _s);
                ::quote::ToTokens::to_tokens(&n_constraints, &mut _s);
                _s
            });
            ::quote::__private::push_eq(&mut _s);
            ::quote::__private::push_group(&mut _s, ::quote::__private::Delimiter::Bracket, {
                let mut _s = ::quote::__private::TokenStream::new();
                {
                    use ::quote::__private::ext::*;
                    let mut _i = 0usize;
                    let has_iter = ::quote::__private::ThereIsNoIteratorInRepetition;
                    #[allow(unused_mut)]
                    let (mut c_degs, i) = c_degs.quote_into_iter();
                    let has_iter = has_iter | i;
                    let _: ::quote::__private::HasIterator = has_iter;
                    while true {
                        let c_degs = match c_degs.next() {
                            Some(_x) => ::quote::__private::RepInterp(_x),
                            None => break,
                        };
                        if _i > 0 {
                            ::quote::__private::push_comma(&mut _s);
                        }
                        _i += 1;
                        ::quote::ToTokens::to_tokens(&c_degs, &mut _s);
                        ::quote::__private::push_ident(&mut _s, "as");
                        ::quote::__private::push_ident(&mut _s, "usize");
                    }
                }
                _s
            });
            ::quote::__private::push_semi(&mut _s);
            ::quote::__private::push_ident(&mut _s, "pub");
            ::quote::__private::push_ident(&mut _s, "fn");
            ::quote::__private::push_ident(&mut _s, "evaluate_transitions");
            ::quote::__private::push_lt(&mut _s);
            ::quote::__private::push_ident(&mut _s, "E");
            ::quote::__private::push_colon(&mut _s);
            ::quote::__private::push_ident(&mut _s, "FieldElement");
            ::quote::__private::push_gt(&mut _s);
            ::quote::__private::push_group(&mut _s, ::quote::__private::Delimiter::Parenthesis, {
                let mut _s = ::quote::__private::TokenStream::new();
                ::quote::__private::push_ident(&mut _s, "frame");
                ::quote::__private::push_colon(&mut _s);
                ::quote::__private::push_and(&mut _s);
                ::quote::__private::push_ident(&mut _s, "EvaluationFrame");
                ::quote::__private::push_lt(&mut _s);
                ::quote::__private::push_ident(&mut _s, "E");
                ::quote::__private::push_gt(&mut _s);
                ::quote::__private::push_comma(&mut _s);
                ::quote::__private::push_ident(&mut _s, "periodic_values");
                ::quote::__private::push_colon(&mut _s);
                ::quote::__private::push_and(&mut _s);
                ::quote::__private::push_group(&mut _s, ::quote::__private::Delimiter::Bracket, {
                    let mut _s = ::quote::__private::TokenStream::new();
                    ::quote::__private::push_ident(&mut _s, "E");
                    _s
                });
                ::quote::__private::push_comma(&mut _s);
                ::quote::__private::push_ident(&mut _s, "result");
                ::quote::__private::push_colon(&mut _s);
                ::quote::__private::push_and(&mut _s);
                ::quote::__private::push_ident(&mut _s, "mut");
                ::quote::__private::push_group(&mut _s, ::quote::__private::Delimiter::Bracket, {
                    let mut _s = ::quote::__private::TokenStream::new();
                    ::quote::__private::push_ident(&mut _s, "E");
                    _s
                });
                ::quote::__private::push_comma(&mut _s);
                _s
            });
            ::quote::__private::push_rarrow(&mut _s);
            ::quote::__private::push_ident(&mut _s, "usize");
            ::quote::__private::push_group(&mut _s, ::quote::__private::Delimiter::Brace, {
                let mut _s = ::quote::__private::TokenStream::new();
                ::quote::__private::push_ident(&mut _s, "let");
                ::quote::__private::push_ident(&mut _s, "current");
                ::quote::__private::push_eq(&mut _s);
                ::quote::__private::push_ident(&mut _s, "frame");
                ::quote::__private::push_dot(&mut _s);
                ::quote::__private::push_ident(&mut _s, "current");
                ::quote::__private::push_group(
                    &mut _s,
                    ::quote::__private::Delimiter::Parenthesis,
                    ::quote::__private::TokenStream::new(),
                );
                ::quote::__private::push_semi(&mut _s);
                ::quote::__private::push_ident(&mut _s, "let");
                ::quote::__private::push_ident(&mut _s, "next");
                ::quote::__private::push_eq(&mut _s);
                ::quote::__private::push_ident(&mut _s, "frame");
                ::quote::__private::push_dot(&mut _s);
                ::quote::__private::push_ident(&mut _s, "next");
                ::quote::__private::push_group(
                    &mut _s,
                    ::quote::__private::Delimiter::Parenthesis,
                    ::quote::__private::TokenStream::new(),
                );
                ::quote::__private::push_semi(&mut _s);
                ::quote::__private::push_ident(&mut _s, "let");
                ::quote::__private::push_ident(&mut _s, "is_body");
                ::quote::__private::push_eq(&mut _s);
                ::quote::__private::push_ident(&mut _s, "current");
                ::quote::__private::push_group(&mut _s, ::quote::__private::Delimiter::Bracket, {
                    let mut _s = ::quote::__private::TokenStream::new();
                    ::quote::__private::push_ident(&mut _s, "BODY");
                    _s
                });
                ::quote::__private::push_semi(&mut _s);
                ::quote::__private::push_ident(&mut _s, "let");
                ::quote::__private::push_ident(&mut _s, "op_flag");
                ::quote::__private::push_eq(&mut _s);
                ::quote::__private::push_ident(&mut _s, "op_flag");
                ::quote::__private::push_group(
                    &mut _s,
                    ::quote::__private::Delimiter::Parenthesis,
                    {
                        let mut _s = ::quote::__private::TokenStream::new();
                        ::quote::__private::push_and(&mut _s);
                        ::quote::__private::push_ident(&mut _s, "current");
                        ::quote::__private::push_group(
                            &mut _s,
                            ::quote::__private::Delimiter::Bracket,
                            {
                                let mut _s = ::quote::__private::TokenStream::new();
                                ::quote::__private::push_ident(&mut _s, "OPCODE_END");
                                ::quote::__private::push_dot2(&mut _s);
                                ::quote::__private::push_ident(&mut _s, "OPCODE_END");
                                ::quote::__private::push_add(&mut _s);
                                ::quote::__private::parse(&mut _s, "7");
                                _s
                            },
                        );
                        _s
                    },
                );
                ::quote::__private::push_star(&mut _s);
                ::quote::__private::push_ident(&mut _s, "current");
                ::quote::__private::push_group(&mut _s, ::quote::__private::Delimiter::Bracket, {
                    let mut _s = ::quote::__private::TokenStream::new();
                    ::quote::__private::push_ident(&mut _s, "BODY");
                    _s
                });
                ::quote::__private::push_star(&mut _s);
                ::quote::__private::push_ident(&mut _s, "funct3_flag");
                ::quote::__private::push_group(
                    &mut _s,
                    ::quote::__private::Delimiter::Parenthesis,
                    {
                        let mut _s = ::quote::__private::TokenStream::new();
                        ::quote::__private::push_and(&mut _s);
                        ::quote::__private::push_ident(&mut _s, "current");
                        ::quote::__private::push_group(
                            &mut _s,
                            ::quote::__private::Delimiter::Bracket,
                            {
                                let mut _s = ::quote::__private::TokenStream::new();
                                ::quote::__private::push_ident(&mut _s, "FUNCT3_END");
                                ::quote::__private::push_dot2(&mut _s);
                                ::quote::__private::push_ident(&mut _s, "FUNCT3_END");
                                ::quote::__private::push_add(&mut _s);
                                ::quote::__private::parse(&mut _s, "3");
                                _s
                            },
                        );
                        _s
                    },
                );
                ::quote::__private::push_semi(&mut _s);
                ::quote::ToTokens::to_tokens(&n_constraints, &mut _s);
                _s
            });
            ::quote::__private::push_ident(&mut _s, "pub");
            ::quote::__private::push_ident(&mut _s, "fn");
            ::quote::__private::push_ident(&mut _s, "constraint_degrees");
            ::quote::__private::push_group(
                &mut _s,
                ::quote::__private::Delimiter::Parenthesis,
                ::quote::__private::TokenStream::new(),
            );
            ::quote::__private::push_rarrow(&mut _s);
            ::quote::__private::push_ident(&mut _s, "Vec");
            ::quote::__private::push_lt(&mut _s);
            ::quote::__private::push_ident(&mut _s, "TransitionConstraintDegree");
            ::quote::__private::push_gt(&mut _s);
            ::quote::__private::push_group(&mut _s, ::quote::__private::Delimiter::Brace, {
                let mut _s = ::quote::__private::TokenStream::new();
                ::quote::__private::push_ident(&mut _s, "let");
                ::quote::__private::push_ident(&mut _s, "mut");
                ::quote::__private::push_ident(&mut _s, "degrees");
                ::quote::__private::push_eq(&mut _s);
                ::quote::__private::push_ident(&mut _s, "Vec");
                ::quote::__private::push_colon2(&mut _s);
                ::quote::__private::push_ident(&mut _s, "with_capacity");
                ::quote::__private::push_group(
                    &mut _s,
                    ::quote::__private::Delimiter::Parenthesis,
                    {
                        let mut _s = ::quote::__private::TokenStream::new();
                        ::quote::__private::push_ident(&mut _s, "TOT_CNT");
                        _s
                    },
                );
                ::quote::__private::push_semi(&mut _s);
                ::quote::__private::push_ident(&mut _s, "for");
                ::quote::__private::push_underscore(&mut _s);
                ::quote::__private::push_ident(&mut _s, "in");
                ::quote::__private::parse(&mut _s, "0");
                ::quote::__private::push_dot2(&mut _s);
                ::quote::__private::push_ident(&mut _s, "TOT_CNT");
                ::quote::__private::push_group(&mut _s, ::quote::__private::Delimiter::Brace, {
                    let mut _s = ::quote::__private::TokenStream::new();
                    {
                        use ::quote::__private::ext::*;
                        let mut _i = 0usize;
                        let has_iter = ::quote::__private::ThereIsNoIteratorInRepetition;
                        #[allow(unused_mut)]
                        let (mut c_degs, i) = c_degs.quote_into_iter();
                        let has_iter = has_iter | i;
                        let _: ::quote::__private::HasIterator = has_iter;
                        while true {
                            let c_degs = match c_degs.next() {
                                Some(_x) => ::quote::__private::RepInterp(_x),
                                None => break,
                            };
                            if _i > 0 {
                                ::quote::__private::push_comma(&mut _s);
                            }
                            _i += 1;
                            ::quote::__private::push_ident(&mut _s, "degrees");
                            ::quote::__private::push_dot(&mut _s);
                            ::quote::__private::push_ident(&mut _s, "push");
                            ::quote::__private::push_group(
                                &mut _s,
                                ::quote::__private::Delimiter::Parenthesis,
                                {
                                    let mut _s = ::quote::__private::TokenStream::new();
                                    ::quote::__private::push_ident(
                                        &mut _s,
                                        "TransitionConstraintDegree",
                                    );
                                    ::quote::__private::push_colon2(&mut _s);
                                    ::quote::__private::push_ident(&mut _s, "new");
                                    ::quote::__private::push_group(
                                        &mut _s,
                                        ::quote::__private::Delimiter::Parenthesis,
                                        {
                                            let mut _s = ::quote::__private::TokenStream::new();
                                            ::quote::__private::push_ident(
                                                &mut _s,
                                                "OPCODE_FLAG_DEG",
                                            );
                                            ::quote::__private::push_add(&mut _s);
                                            ::quote::__private::push_ident(&mut _s, "RD_FLAG_DEG");
                                            ::quote::__private::push_add(&mut _s);
                                            ::quote::__private::push_ident(&mut _s, "RS1_FLAG_DEG");
                                            ::quote::__private::push_add(&mut _s);
                                            ::quote::__private::push_ident(&mut _s, "RS2_FLAG_DEG");
                                            ::quote::__private::push_add(&mut _s);
                                            ::quote::__private::push_ident(
                                                &mut _s,
                                                "FUNCT3_FLAG_DEG",
                                            );
                                            ::quote::__private::push_add(&mut _s);
                                            ::quote::__private::push_ident(
                                                &mut _s,
                                                "BODY_FLAG_DEG",
                                            );
                                            ::quote::__private::push_add(&mut _s);
                                            ::quote::ToTokens::to_tokens(&c_degs, &mut _s);
                                            ::quote::__private::push_comma(&mut _s);
                                            _s
                                        },
                                    );
                                    _s
                                },
                            );
                            ::quote::__private::push_semi(&mut _s);
                            ::quote::__private::push_ident(&mut _s, "as");
                            ::quote::__private::push_ident(&mut _s, "usize");
                        }
                    }
                    _s
                });
                ::quote::__private::push_ident(&mut _s, "degrees");
                _s
            });
            ::quote::__private::push_ident(&mut _s, "fn");
            ::quote::__private::push_ident(&mut _s, "evaluate_constraints");
            ::quote::__private::push_lt(&mut _s);
            ::quote::__private::push_ident(&mut _s, "E");
            ::quote::__private::push_colon(&mut _s);
            ::quote::__private::push_ident(&mut _s, "FieldElement");
            ::quote::__private::push_gt(&mut _s);
            ::quote::__private::push_group(&mut _s, ::quote::__private::Delimiter::Parenthesis, {
                let mut _s = ::quote::__private::TokenStream::new();
                ::quote::__private::push_ident(&mut _s, "current");
                ::quote::__private::push_colon(&mut _s);
                ::quote::__private::push_and(&mut _s);
                ::quote::__private::push_group(&mut _s, ::quote::__private::Delimiter::Bracket, {
                    let mut _s = ::quote::__private::TokenStream::new();
                    ::quote::__private::push_ident(&mut _s, "E");
                    _s
                });
                ::quote::__private::push_comma(&mut _s);
                ::quote::__private::push_ident(&mut _s, "next");
                ::quote::__private::push_colon(&mut _s);
                ::quote::__private::push_and(&mut _s);
                ::quote::__private::push_group(&mut _s, ::quote::__private::Delimiter::Bracket, {
                    let mut _s = ::quote::__private::TokenStream::new();
                    ::quote::__private::push_ident(&mut _s, "E");
                    _s
                });
                ::quote::__private::push_comma(&mut _s);
                ::quote::__private::push_ident(&mut _s, "rd");
                ::quote::__private::push_colon(&mut _s);
                ::quote::__private::push_ident(&mut _s, "usize");
                ::quote::__private::push_comma(&mut _s);
                ::quote::__private::push_ident(&mut _s, "rs1");
                ::quote::__private::push_colon(&mut _s);
                ::quote::__private::push_ident(&mut _s, "usize");
                ::quote::__private::push_comma(&mut _s);
                ::quote::__private::push_ident(&mut _s, "rs2");
                ::quote::__private::push_colon(&mut _s);
                ::quote::__private::push_ident(&mut _s, "usize");
                ::quote::__private::push_comma(&mut _s);
                ::quote::__private::push_ident(&mut _s, "result");
                ::quote::__private::push_colon(&mut _s);
                ::quote::__private::push_and(&mut _s);
                ::quote::__private::push_ident(&mut _s, "mut");
                ::quote::__private::push_group(&mut _s, ::quote::__private::Delimiter::Bracket, {
                    let mut _s = ::quote::__private::TokenStream::new();
                    ::quote::__private::push_ident(&mut _s, "E");
                    _s
                });
                _s
            });
            ::quote::__private::push_group(
                &mut _s,
                ::quote::__private::Delimiter::Brace,
                ::quote::__private::TokenStream::new(),
            );
            ::quote::__private::push_ident(&mut _s, "fn");
            ::quote::__private::push_ident(&mut _s, "funct3_flag");
            ::quote::__private::push_lt(&mut _s);
            ::quote::__private::push_ident(&mut _s, "E");
            ::quote::__private::push_gt(&mut _s);
            ::quote::__private::push_group(&mut _s, ::quote::__private::Delimiter::Parenthesis, {
                let mut _s = ::quote::__private::TokenStream::new();
                ::quote::__private::push_ident(&mut _s, "test");
                ::quote::__private::push_colon(&mut _s);
                ::quote::__private::push_and(&mut _s);
                ::quote::__private::push_group(&mut _s, ::quote::__private::Delimiter::Bracket, {
                    let mut _s = ::quote::__private::TokenStream::new();
                    ::quote::__private::push_ident(&mut _s, "E");
                    _s
                });
                _s
            });
            ::quote::__private::push_rarrow(&mut _s);
            ::quote::__private::push_ident(&mut _s, "E");
            ::quote::__private::push_ident(&mut _s, "where");
            ::quote::__private::push_ident(&mut _s, "E");
            ::quote::__private::push_colon(&mut _s);
            ::quote::__private::push_ident(&mut _s, "FieldElement");
            ::quote::__private::push_comma(&mut _s);
            ::quote::__private::push_group(&mut _s, ::quote::__private::Delimiter::Brace, {
                let mut _s = ::quote::__private::TokenStream::new();
                ::quote::__private::push_ident(&mut _s, "assert_eq");
                ::quote::__private::push_bang(&mut _s);
                ::quote::__private::push_group(
                    &mut _s,
                    ::quote::__private::Delimiter::Parenthesis,
                    {
                        let mut _s = ::quote::__private::TokenStream::new();
                        ::quote::__private::push_ident(&mut _s, "test");
                        ::quote::__private::push_dot(&mut _s);
                        ::quote::__private::push_ident(&mut _s, "len");
                        ::quote::__private::push_group(
                            &mut _s,
                            ::quote::__private::Delimiter::Parenthesis,
                            ::quote::__private::TokenStream::new(),
                        );
                        ::quote::__private::push_comma(&mut _s);
                        ::quote::__private::parse(&mut _s, "3");
                        _s
                    },
                );
                ::quote::__private::push_semi(&mut _s);
                ::quote::ToTokens::to_tokens(&funct3_flag_contents, &mut _s);
                _s
            });
            ::quote::__private::push_ident(&mut _s, "fn");
            ::quote::__private::push_ident(&mut _s, "op_flag");
            ::quote::__private::push_lt(&mut _s);
            ::quote::__private::push_ident(&mut _s, "E");
            ::quote::__private::push_gt(&mut _s);
            ::quote::__private::push_group(&mut _s, ::quote::__private::Delimiter::Parenthesis, {
                let mut _s = ::quote::__private::TokenStream::new();
                ::quote::__private::push_ident(&mut _s, "test");
                ::quote::__private::push_colon(&mut _s);
                ::quote::__private::push_and(&mut _s);
                ::quote::__private::push_group(&mut _s, ::quote::__private::Delimiter::Bracket, {
                    let mut _s = ::quote::__private::TokenStream::new();
                    ::quote::__private::push_ident(&mut _s, "E");
                    _s
                });
                _s
            });
            ::quote::__private::push_rarrow(&mut _s);
            ::quote::__private::push_ident(&mut _s, "E");
            ::quote::__private::push_ident(&mut _s, "where");
            ::quote::__private::push_ident(&mut _s, "E");
            ::quote::__private::push_colon(&mut _s);
            ::quote::__private::push_ident(&mut _s, "FieldElement");
            ::quote::__private::push_comma(&mut _s);
            ::quote::__private::push_group(&mut _s, ::quote::__private::Delimiter::Brace, {
                let mut _s = ::quote::__private::TokenStream::new();
                ::quote::__private::push_ident(&mut _s, "assert_eq");
                ::quote::__private::push_bang(&mut _s);
                ::quote::__private::push_group(
                    &mut _s,
                    ::quote::__private::Delimiter::Parenthesis,
                    {
                        let mut _s = ::quote::__private::TokenStream::new();
                        ::quote::__private::push_ident(&mut _s, "test");
                        ::quote::__private::push_dot(&mut _s);
                        ::quote::__private::push_ident(&mut _s, "len");
                        ::quote::__private::push_group(
                            &mut _s,
                            ::quote::__private::Delimiter::Parenthesis,
                            ::quote::__private::TokenStream::new(),
                        );
                        ::quote::__private::push_comma(&mut _s);
                        ::quote::__private::parse(&mut _s, "7");
                        _s
                    },
                );
                ::quote::__private::push_semi(&mut _s);
                ::quote::__private::push_ident(&mut _s, "binary_flag");
                ::quote::__private::push_group(
                    &mut _s,
                    ::quote::__private::Delimiter::Parenthesis,
                    {
                        let mut _s = ::quote::__private::TokenStream::new();
                        ::quote::__private::push_and(&mut _s);
                        ::quote::ToTokens::to_tokens(&opcode, &mut _s);
                        ::quote::__private::push_comma(&mut _s);
                        ::quote::__private::push_ident(&mut _s, "test");
                        ::quote::__private::push_comma(&mut _s);
                        ::quote::__private::push_ident(&mut _s, "E");
                        ::quote::__private::push_colon2(&mut _s);
                        ::quote::__private::push_ident(&mut _s, "ONE");
                        _s
                    },
                );
                _s
            });
            ::quote::__private::push_ident(&mut _s, "fn");
            ::quote::__private::push_ident(&mut _s, "reg_flag");
            ::quote::__private::push_lt(&mut _s);
            ::quote::__private::push_ident(&mut _s, "E");
            ::quote::__private::push_gt(&mut _s);
            ::quote::__private::push_group(&mut _s, ::quote::__private::Delimiter::Parenthesis, {
                let mut _s = ::quote::__private::TokenStream::new();
                ::quote::__private::push_ident(&mut _s, "reg");
                ::quote::__private::push_colon(&mut _s);
                ::quote::__private::push_ident(&mut _s, "u8");
                ::quote::__private::push_comma(&mut _s);
                ::quote::__private::push_ident(&mut _s, "test");
                ::quote::__private::push_colon(&mut _s);
                ::quote::__private::push_and(&mut _s);
                ::quote::__private::push_group(&mut _s, ::quote::__private::Delimiter::Bracket, {
                    let mut _s = ::quote::__private::TokenStream::new();
                    ::quote::__private::push_ident(&mut _s, "E");
                    _s
                });
                _s
            });
            ::quote::__private::push_rarrow(&mut _s);
            ::quote::__private::push_ident(&mut _s, "E");
            ::quote::__private::push_ident(&mut _s, "where");
            ::quote::__private::push_ident(&mut _s, "E");
            ::quote::__private::push_colon(&mut _s);
            ::quote::__private::push_ident(&mut _s, "FieldElement");
            ::quote::__private::push_comma(&mut _s);
            ::quote::__private::push_group(&mut _s, ::quote::__private::Delimiter::Brace, {
                let mut _s = ::quote::__private::TokenStream::new();
                ::quote::__private::push_ident(&mut _s, "assert_eq");
                ::quote::__private::push_bang(&mut _s);
                ::quote::__private::push_group(
                    &mut _s,
                    ::quote::__private::Delimiter::Parenthesis,
                    {
                        let mut _s = ::quote::__private::TokenStream::new();
                        ::quote::__private::push_ident(&mut _s, "test");
                        ::quote::__private::push_dot(&mut _s);
                        ::quote::__private::push_ident(&mut _s, "len");
                        ::quote::__private::push_group(
                            &mut _s,
                            ::quote::__private::Delimiter::Parenthesis,
                            ::quote::__private::TokenStream::new(),
                        );
                        ::quote::__private::push_comma(&mut _s);
                        ::quote::__private::parse(&mut _s, "5");
                        _s
                    },
                );
                ::quote::__private::push_semi(&mut _s);
                ::quote::__private::push_ident(&mut _s, "binary_flag");
                ::quote::__private::push_group(
                    &mut _s,
                    ::quote::__private::Delimiter::Parenthesis,
                    {
                        let mut _s = ::quote::__private::TokenStream::new();
                        ::quote::__private::push_and(&mut _s);
                        ::quote::__private::push_ident(&mut _s, "to_binary");
                        ::quote::__private::push_group(
                            &mut _s,
                            ::quote::__private::Delimiter::Parenthesis,
                            {
                                let mut _s = ::quote::__private::TokenStream::new();
                                ::quote::__private::push_ident(&mut _s, "reg");
                                ::quote::__private::push_comma(&mut _s);
                                ::quote::__private::push_ident(&mut _s, "E");
                                ::quote::__private::push_colon2(&mut _s);
                                ::quote::__private::push_ident(&mut _s, "ZERO");
                                ::quote::__private::push_comma(&mut _s);
                                ::quote::__private::push_ident(&mut _s, "E");
                                ::quote::__private::push_colon2(&mut _s);
                                ::quote::__private::push_ident(&mut _s, "ONE");
                                _s
                            },
                        );
                        ::quote::__private::push_comma(&mut _s);
                        ::quote::__private::push_ident(&mut _s, "test");
                        ::quote::__private::push_comma(&mut _s);
                        ::quote::__private::push_ident(&mut _s, "E");
                        ::quote::__private::push_colon2(&mut _s);
                        ::quote::__private::push_ident(&mut _s, "ONE");
                        _s
                    },
                );
                _s
            });
            ::quote::__private::push_ident(&mut _s, "pub");
            ::quote::__private::push_ident(&mut _s, "fn");
            ::quote::__private::push_ident(&mut _s, "binary_flag");
            ::quote::__private::push_lt(&mut _s);
            ::quote::__private::push_ident(&mut _s, "E");
            ::quote::__private::push_gt(&mut _s);
            ::quote::__private::push_group(&mut _s, ::quote::__private::Delimiter::Parenthesis, {
                let mut _s = ::quote::__private::TokenStream::new();
                ::quote::__private::push_ident(&mut _s, "expected");
                ::quote::__private::push_colon(&mut _s);
                ::quote::__private::push_and(&mut _s);
                ::quote::__private::push_group(&mut _s, ::quote::__private::Delimiter::Bracket, {
                    let mut _s = ::quote::__private::TokenStream::new();
                    ::quote::__private::push_ident(&mut _s, "E");
                    _s
                });
                ::quote::__private::push_comma(&mut _s);
                ::quote::__private::push_ident(&mut _s, "test");
                ::quote::__private::push_colon(&mut _s);
                ::quote::__private::push_and(&mut _s);
                ::quote::__private::push_group(&mut _s, ::quote::__private::Delimiter::Bracket, {
                    let mut _s = ::quote::__private::TokenStream::new();
                    ::quote::__private::push_ident(&mut _s, "E");
                    _s
                });
                ::quote::__private::push_comma(&mut _s);
                ::quote::__private::push_ident(&mut _s, "one");
                ::quote::__private::push_colon(&mut _s);
                ::quote::__private::push_ident(&mut _s, "E");
                _s
            });
            ::quote::__private::push_rarrow(&mut _s);
            ::quote::__private::push_ident(&mut _s, "E");
            ::quote::__private::push_ident(&mut _s, "where");
            ::quote::__private::push_ident(&mut _s, "E");
            ::quote::__private::push_colon(&mut _s);
            ::quote::__private::push_ident(&mut _s, "Mul");
            ::quote::__private::push_lt(&mut _s);
            ::quote::__private::push_ident(&mut _s, "Output");
            ::quote::__private::push_eq(&mut _s);
            ::quote::__private::push_ident(&mut _s, "E");
            ::quote::__private::push_gt(&mut _s);
            ::quote::__private::push_add(&mut _s);
            ::quote::__private::push_ident(&mut _s, "Sub");
            ::quote::__private::push_lt(&mut _s);
            ::quote::__private::push_ident(&mut _s, "Output");
            ::quote::__private::push_eq(&mut _s);
            ::quote::__private::push_ident(&mut _s, "E");
            ::quote::__private::push_gt(&mut _s);
            ::quote::__private::push_add(&mut _s);
            ::quote::__private::push_ident(&mut _s, "Copy");
            ::quote::__private::push_add(&mut _s);
            ::quote::__private::push_ident(&mut _s, "FieldElement");
            ::quote::__private::push_comma(&mut _s);
            ::quote::__private::push_group(&mut _s, ::quote::__private::Delimiter::Brace, {
                let mut _s = ::quote::__private::TokenStream::new();
                ::quote::__private::push_ident(&mut _s, "let");
                ::quote::__private::push_ident(&mut _s, "mut");
                ::quote::__private::push_ident(&mut _s, "result");
                ::quote::__private::push_eq(&mut _s);
                ::quote::__private::push_ident(&mut _s, "one");
                ::quote::__private::push_semi(&mut _s);
                ::quote::__private::push_ident(&mut _s, "for");
                ::quote::__private::push_group(
                    &mut _s,
                    ::quote::__private::Delimiter::Parenthesis,
                    {
                        let mut _s = ::quote::__private::TokenStream::new();
                        ::quote::__private::push_ident(&mut _s, "i");
                        ::quote::__private::push_comma(&mut _s);
                        ::quote::__private::push_ident(&mut _s, "bit");
                        _s
                    },
                );
                ::quote::__private::push_ident(&mut _s, "in");
                ::quote::__private::push_ident(&mut _s, "expected");
                ::quote::__private::push_dot(&mut _s);
                ::quote::__private::push_ident(&mut _s, "iter");
                ::quote::__private::push_group(
                    &mut _s,
                    ::quote::__private::Delimiter::Parenthesis,
                    ::quote::__private::TokenStream::new(),
                );
                ::quote::__private::push_dot(&mut _s);
                ::quote::__private::push_ident(&mut _s, "enumerate");
                ::quote::__private::push_group(
                    &mut _s,
                    ::quote::__private::Delimiter::Parenthesis,
                    ::quote::__private::TokenStream::new(),
                );
                ::quote::__private::push_group(&mut _s, ::quote::__private::Delimiter::Brace, {
                    let mut _s = ::quote::__private::TokenStream::new();
                    ::quote::__private::push_ident(&mut _s, "result");
                    ::quote::__private::push_mul_eq(&mut _s);
                    ::quote::__private::push_ident(&mut _s, "if");
                    ::quote::__private::push_ident(&mut _s, "bit");
                    ::quote::__private::push_eq_eq(&mut _s);
                    ::quote::__private::push_and(&mut _s);
                    ::quote::__private::push_ident(&mut _s, "one");
                    ::quote::__private::push_group(
                        &mut _s,
                        ::quote::__private::Delimiter::Brace,
                        {
                            let mut _s = ::quote::__private::TokenStream::new();
                            ::quote::__private::push_ident(&mut _s, "test");
                            ::quote::__private::push_group(
                                &mut _s,
                                ::quote::__private::Delimiter::Bracket,
                                {
                                    let mut _s = ::quote::__private::TokenStream::new();
                                    ::quote::__private::push_ident(&mut _s, "i");
                                    _s
                                },
                            );
                            _s
                        },
                    );
                    ::quote::__private::push_ident(&mut _s, "else");
                    ::quote::__private::push_group(
                        &mut _s,
                        ::quote::__private::Delimiter::Brace,
                        {
                            let mut _s = ::quote::__private::TokenStream::new();
                            ::quote::__private::push_ident(&mut _s, "one");
                            ::quote::__private::push_sub(&mut _s);
                            ::quote::__private::push_ident(&mut _s, "test");
                            ::quote::__private::push_group(
                                &mut _s,
                                ::quote::__private::Delimiter::Bracket,
                                {
                                    let mut _s = ::quote::__private::TokenStream::new();
                                    ::quote::__private::push_ident(&mut _s, "i");
                                    _s
                                },
                            );
                            _s
                        },
                    );
                    ::quote::__private::push_semi(&mut _s);
                    _s
                });
                ::quote::__private::push_ident(&mut _s, "result");
                _s
            });
            ::quote::__private::push_ident(&mut _s, "fn");
            ::quote::__private::push_ident(&mut _s, "to_binary");
            ::quote::__private::push_lt(&mut _s);
            ::quote::__private::push_ident(&mut _s, "E");
            ::quote::__private::push_colon(&mut _s);
            ::quote::__private::push_ident(&mut _s, "Copy");
            ::quote::__private::push_gt(&mut _s);
            ::quote::__private::push_group(&mut _s, ::quote::__private::Delimiter::Parenthesis, {
                let mut _s = ::quote::__private::TokenStream::new();
                ::quote::__private::push_ident(&mut _s, "reg");
                ::quote::__private::push_colon(&mut _s);
                ::quote::__private::push_ident(&mut _s, "u8");
                ::quote::__private::push_comma(&mut _s);
                ::quote::__private::push_ident(&mut _s, "zero");
                ::quote::__private::push_colon(&mut _s);
                ::quote::__private::push_ident(&mut _s, "E");
                ::quote::__private::push_comma(&mut _s);
                ::quote::__private::push_ident(&mut _s, "one");
                ::quote::__private::push_colon(&mut _s);
                ::quote::__private::push_ident(&mut _s, "E");
                _s
            });
            ::quote::__private::push_rarrow(&mut _s);
            ::quote::__private::push_group(&mut _s, ::quote::__private::Delimiter::Bracket, {
                let mut _s = ::quote::__private::TokenStream::new();
                ::quote::__private::push_ident(&mut _s, "E");
                ::quote::__private::push_semi(&mut _s);
                ::quote::__private::parse(&mut _s, "5");
                _s
            });
            ::quote::__private::push_group(&mut _s, ::quote::__private::Delimiter::Brace, {
                let mut _s = ::quote::__private::TokenStream::new();
                ::quote::__private::push_ident(&mut _s, "let");
                ::quote::__private::push_ident(&mut _s, "mut");
                ::quote::__private::push_ident(&mut _s, "result");
                ::quote::__private::push_eq(&mut _s);
                ::quote::__private::push_group(&mut _s, ::quote::__private::Delimiter::Bracket, {
                    let mut _s = ::quote::__private::TokenStream::new();
                    ::quote::__private::push_ident(&mut _s, "zero");
                    ::quote::__private::push_semi(&mut _s);
                    ::quote::__private::parse(&mut _s, "5");
                    _s
                });
                ::quote::__private::push_semi(&mut _s);
                ::quote::__private::push_ident(&mut _s, "for");
                ::quote::__private::push_ident(&mut _s, "i");
                ::quote::__private::push_ident(&mut _s, "in");
                ::quote::__private::parse(&mut _s, "5");
                ::quote::__private::push_dot2(&mut _s);
                ::quote::__private::parse(&mut _s, "0");
                ::quote::__private::push_group(&mut _s, ::quote::__private::Delimiter::Brace, {
                    let mut _s = ::quote::__private::TokenStream::new();
                    ::quote::__private::push_ident(&mut _s, "if");
                    ::quote::__private::push_ident(&mut _s, "reg");
                    ::quote::__private::push_and(&mut _s);
                    ::quote::__private::push_group(
                        &mut _s,
                        ::quote::__private::Delimiter::Parenthesis,
                        {
                            let mut _s = ::quote::__private::TokenStream::new();
                            ::quote::__private::parse(&mut _s, "1");
                            ::quote::__private::push_shl(&mut _s);
                            ::quote::__private::push_ident(&mut _s, "i");
                            _s
                        },
                    );
                    ::quote::__private::push_ne(&mut _s);
                    ::quote::__private::parse(&mut _s, "0");
                    ::quote::__private::push_group(
                        &mut _s,
                        ::quote::__private::Delimiter::Brace,
                        {
                            let mut _s = ::quote::__private::TokenStream::new();
                            ::quote::__private::push_ident(&mut _s, "result");
                            ::quote::__private::push_group(
                                &mut _s,
                                ::quote::__private::Delimiter::Bracket,
                                {
                                    let mut _s = ::quote::__private::TokenStream::new();
                                    ::quote::__private::push_ident(&mut _s, "i");
                                    _s
                                },
                            );
                            ::quote::__private::push_eq(&mut _s);
                            ::quote::__private::push_ident(&mut _s, "one");
                            ::quote::__private::push_semi(&mut _s);
                            _s
                        },
                    );
                    _s
                });
                ::quote::__private::push_ident(&mut _s, "result");
                _s
            });
            ::quote::__private::push_ident(&mut _s, "fn");
            ::quote::__private::push_ident(&mut _s, "get_immediate");
            ::quote::__private::push_lt(&mut _s);
            ::quote::__private::push_ident(&mut _s, "E");
            ::quote::__private::push_colon(&mut _s);
            ::quote::__private::push_ident(&mut _s, "FieldElement");
            ::quote::__private::push_gt(&mut _s);
            ::quote::__private::push_group(&mut _s, ::quote::__private::Delimiter::Parenthesis, {
                let mut _s = ::quote::__private::TokenStream::new();
                ::quote::__private::push_ident(&mut _s, "op");
                ::quote::__private::push_colon(&mut _s);
                ::quote::__private::push_and(&mut _s);
                ::quote::__private::push_group(&mut _s, ::quote::__private::Delimiter::Bracket, {
                    let mut _s = ::quote::__private::TokenStream::new();
                    ::quote::__private::push_ident(&mut _s, "E");
                    _s
                });
                _s
            });
            ::quote::__private::push_rarrow(&mut _s);
            ::quote::__private::push_ident(&mut _s, "E");
            ::quote::__private::push_group(&mut _s, ::quote::__private::Delimiter::Brace, {
                let mut _s = ::quote::__private::TokenStream::new();
                ::quote::__private::push_ident(&mut _s, "let");
                ::quote::__private::push_ident(&mut _s, "mut");
                ::quote::__private::push_ident(&mut _s, "result");
                ::quote::__private::push_eq(&mut _s);
                ::quote::__private::push_ident(&mut _s, "E");
                ::quote::__private::push_colon2(&mut _s);
                ::quote::__private::push_ident(&mut _s, "ZERO");
                ::quote::__private::push_semi(&mut _s);
                ::quote::__private::push_ident(&mut _s, "assert_eq");
                ::quote::__private::push_bang(&mut _s);
                ::quote::__private::push_group(
                    &mut _s,
                    ::quote::__private::Delimiter::Parenthesis,
                    {
                        let mut _s = ::quote::__private::TokenStream::new();
                        ::quote::__private::push_ident(&mut _s, "op");
                        ::quote::__private::push_dot(&mut _s);
                        ::quote::__private::push_ident(&mut _s, "len");
                        ::quote::__private::push_group(
                            &mut _s,
                            ::quote::__private::Delimiter::Parenthesis,
                            ::quote::__private::TokenStream::new(),
                        );
                        ::quote::__private::push_comma(&mut _s);
                        ::quote::__private::parse(&mut _s, "12");
                        _s
                    },
                );
                ::quote::__private::push_semi(&mut _s);
                ::quote::__private::push_ident(&mut _s, "for");
                ::quote::__private::push_group(
                    &mut _s,
                    ::quote::__private::Delimiter::Parenthesis,
                    {
                        let mut _s = ::quote::__private::TokenStream::new();
                        ::quote::__private::push_ident(&mut _s, "i");
                        ::quote::__private::push_comma(&mut _s);
                        ::quote::__private::push_ident(&mut _s, "bit");
                        _s
                    },
                );
                ::quote::__private::push_ident(&mut _s, "in");
                ::quote::__private::push_ident(&mut _s, "op");
                ::quote::__private::push_dot(&mut _s);
                ::quote::__private::push_ident(&mut _s, "iter");
                ::quote::__private::push_group(
                    &mut _s,
                    ::quote::__private::Delimiter::Parenthesis,
                    ::quote::__private::TokenStream::new(),
                );
                ::quote::__private::push_dot(&mut _s);
                ::quote::__private::push_ident(&mut _s, "enumerate");
                ::quote::__private::push_group(
                    &mut _s,
                    ::quote::__private::Delimiter::Parenthesis,
                    ::quote::__private::TokenStream::new(),
                );
                ::quote::__private::push_group(&mut _s, ::quote::__private::Delimiter::Brace, {
                    let mut _s = ::quote::__private::TokenStream::new();
                    ::quote::__private::push_ident(&mut _s, "result");
                    ::quote::__private::push_add_eq(&mut _s);
                    ::quote::__private::push_star(&mut _s);
                    ::quote::__private::push_ident(&mut _s, "bit");
                    ::quote::__private::push_star(&mut _s);
                    ::quote::__private::push_ident(&mut _s, "E");
                    ::quote::__private::push_colon2(&mut _s);
                    ::quote::__private::push_ident(&mut _s, "from");
                    ::quote::__private::push_group(
                        &mut _s,
                        ::quote::__private::Delimiter::Parenthesis,
                        {
                            let mut _s = ::quote::__private::TokenStream::new();
                            ::quote::__private::parse(&mut _s, "1u32");
                            ::quote::__private::push_shl(&mut _s);
                            ::quote::__private::push_ident(&mut _s, "i");
                            _s
                        },
                    );
                    ::quote::__private::push_semi(&mut _s);
                    _s
                });
                ::quote::__private::push_ident(&mut _s, "result");
                _s
            });
            _s
        });
        _s
    }
    .into()
}
const _: () = {
    extern crate proc_macro;
    #[rustc_proc_macro_decls]
    #[used]
    #[allow(deprecated)]
    static _DECLS: &[proc_macro::bridge::client::ProcMacro] =
        &[proc_macro::bridge::client::ProcMacro::bang("air", air)];
};
