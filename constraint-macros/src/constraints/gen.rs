extern crate proc_macro;
use crate::constraints::parse::{Air, Field};
use proc_macro::TokenStream;
use quote::quote;

use crate::{REG_BITS, REG_NUM_PO2, SHAMT_BITS};

fn make_flag<'a, const VAL_LOG2: usize>(
    fields: impl IntoIterator<Item = &'a Field>,
    item: Field,
) -> (proc_macro2::TokenStream, usize) {
    if fields.into_iter().any(|f| *f == item) {
        (
            quote! { binary_flag(&to_binary::<#VAL_LOG2, _>(val, E::ZERO, E::ONE), test, E::ONE) }
                .into(),
            VAL_LOG2,
        )
    } else {
        (quote! { E::ONE }.into(), 0)
    }
}

pub fn generate(config: Air) -> TokenStream {
    let Air {
        name,
        opcode,
        funct3,
        parse,
        constraints,
    } = config;
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

    let (rs1_flag_contents, rs1_deg) = make_flag::<'_, REG_NUM_PO2>(&parse, Field::Rs1);
    let (rs2_flag_contents, rs2_deg) = make_flag::<'_, REG_NUM_PO2>(&parse, Field::Rs2);
    let (rd_flag_contents, rd_deg) = make_flag::<'_, REG_NUM_PO2>(&parse, Field::Rd);
    let (shamt_flag_contents, shamt_deg) = make_flag::<'_, SHAMT_BITS>(&parse, Field::Shamt);

    let c_exprs = constraints
        .iter()
        .flat_map(|c| c.clone().to_token_stream::<REG_BITS>().into_iter());
    let c_degs = constraints
        .iter()
        .flat_map(|c| c.degree::<REG_BITS>().into_iter());
    let n_constraints = constraints
        .iter()
        .flat_map(|c| c.degree::<REG_BITS>().into_iter())
        .count();

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
                const RD_CNT: usize = 1 << #rd_deg as usize - 1;
                const RS1_CNT: usize = 1 << #rs1_deg as usize;
                const RS2_CNT: usize = 1 << #rs2_deg as usize;
                const SHAMT_CNT: usize = 1 << #shamt_deg as usize;
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
                    assert_eq!(result.len(), constraint_degrees().len(), "result length does not match constraint degrees length");

                    for rrd in 1..=RD_CNT {
                        for rs1 in 0..RS1_CNT {
                            for rs2 in 0..RS2_CNT {
                                for shamt in 0..SHAMT_CNT {

                                    let rd_flag = rd_flag(rrd as u8, &current[RD_END..RD_END + 5]);
                                    let rs1_flag = rs1_flag(rs1 as u8, &current[RS1_END..RS1_END + 5]);
                                    let rs2_flag = rs2_flag(rs2 as u8, &current[RS2_END..RS2_END + 5]);
                                    let funct3_flag = funct3_flag(&current[FUNCT3_END..FUNCT3_END + 3]);
                                    let shamt_flag = shamt_flag(shamt as u8, &current[SHAMT_END..SHAMT_END + 5]);
                                    let op_flag = op_flag(&current[OPCODE_END..OPCODE_END + 7]);
                                    let body_flag = current[BODY];

                                    let cumulative_flag = op_flag * rd_flag * rs1_flag * rs2_flag * body_flag * funct3_flag * shamt_flag;
                                    // TODO: fix sign
                                    let simm = get_immediate(&current[UIMM_END..UIMM_END + 12]);
                                    let uimm = get_immediate(&current[UIMM_END..UIMM_END + 12]);
                                    let upper_imm = get_upper_immediate(&current[UIMM_END..UIMM_END + 20]);
                                    let pc = current[PC];
                                    let h0 = current[H_0];
                                    let h1 = current[H_1];
                                    let h2 = current[H_2];
                                    let h3 = current[H_3];
                                    let h4 = current[H_4];
                                    let h5 = current[H_5];
                                    let rd = next[REGISTER_START + rrd];
                                    let rs1 = current[REGISTER_START + rs1];
                                    let rs2 = current[REGISTER_START + rs2];

                                    #(
                                        result[index] = (#c_exprs) * cumulative_flag;
                                        index += 1;
                                    )*
                                    if current[CYCLE] == E::ONE ||  current[CYCLE] == E::ZERO {
                                        assert_eq!(result[0], E::ZERO, "constraint {:?} rd value: {rd} rd: {rrd} flag: {rd_flag} cycle: {} {:?}", &current[RD_END..RD_END + 5],current[CYCLE], &current[UIMM_END..UIMM_END + 20]);
                                    }
                                    
                                }
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

                fn rd_flag<E: FieldElement>(val: u8, test: &[E]) -> E {
                    assert_eq!(test.len(), 5, "requested rd flag with invalid length {}", test.len());
                    #rd_flag_contents
                }

                fn rs1_flag<E: FieldElement>(val: u8, test: &[E]) -> E {
                    assert_eq!(test.len(), 5, "requested rs1 flag with invalid length {}", test.len());
                    #rs1_flag_contents
                }

                fn rs2_flag<E: FieldElement>(val: u8, test: &[E]) -> E {
                    assert_eq!(test.len(), 5, "requested rs2 flag with invalid length {}", test.len());
                    #rs2_flag_contents
                }

                fn funct3_flag<E>(test: &[E]) -> E
                where
                    E: FieldElement,
                {
                    assert_eq!(test.len(), 3, "requested funct3 flag with invalid length {}", test.len());
                    #funct3_flag_contents
                }

                fn op_flag<E>(test: &[E]) -> E
                where
                    E: FieldElement,
                {

                    assert_eq!(test.len(), 7, "requested op flag with invalid length {}", test.len());
                    binary_flag(&#opcode, test, E::ONE)
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

                fn to_binary<const M: usize, E: Copy>(reg: u8, zero: E, one: E) -> [E; M] {
                    let mut result = [zero; M];
                    assert!(reg < (1 << M), "requested binary representation of value({reg}) bigger than output array({M})");
                    for i in 0..M {
                        if reg & (1 << i) != 0 {
                            result[M - i - 1] = one;
                        }
                    }

                    result
                }

                fn shamt_flag<E: FieldElement>(val: u8, test: &[E]) -> E {
                    assert_eq!(test.len(), 5, "requested shamt flag with invalid length {}", test.len());
                    #shamt_flag_contents
                }

                // Degree: 1
                // TODO: this is unsigned, but we need signed
                fn get_immediate<E: FieldElement>(op: &[E]) -> E {
                    let mut result = E::ZERO;
                    assert_eq!(op.len(), 12, "requested upper immediate with invalid length {}", op.len());
                    for (i, bit) in op.iter().enumerate() {
                        result += *bit * E::from(1u32 << i);
                    }
                    result
                }

                fn get_upper_immediate<E: FieldElement>(op: &[E]) -> E {
                    let mut result = E::ZERO;
                    assert_eq!(op.len(), 20, "requested upper immediate with invalid length {}", op.len());
                    for (i, bit) in op.iter().rev().enumerate() {
                        result += *bit * E::from(1u32 << (i + 12));
                    }
                    result
                }
        }
    }.into()
}
