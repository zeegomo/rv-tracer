// extern crate proc_macro;
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
            quote! { binary_flag(&to_binary::<#VAL_LOG2, _>(val, E::ZERO, E::ONE), test, E::ONE) },
            VAL_LOG2,
        )
    } else {
        (quote! { E::ONE }, 0)
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
                use winterfell::{EvaluationFrame, TransitionConstraintDegree, math::{FieldElement, StarkField}};

                const OPCODE_FLAG_DEG: usize = 7;
                const REG_BITS: usize = #REG_BITS;
                const FUNCT3_FLAG_DEG: usize = #funct3_deg as usize;
                const SHAMT_CNT: usize = 1 << #shamt_deg as usize;
                const SHAMT_DEG: usize = #shamt_deg as usize;
                const TOT_CNT: usize = SHAMT_CNT;
                const BODY_FLAG_DEG: usize = 1;
                const CONSTRAINT_DEGS: [usize; #n_constraints] = [#(#c_degs as usize),*];

                pub fn evaluate_transitions<E: FieldElement>(
                    frame: &EvaluationFrame<E>,
                    periodic_values: &[E],
                    result: &mut [E],
                ) -> usize {
                    let current = frame.current();
                    let next = frame.next();
                    let mut index = 0;

                    let body_flag = current[BODY];
                    let funct3_flag = funct3_flag(&current[FUNCT3_END..FUNCT3_END + 3]);
                    let op_flag = op_flag(&current[OPCODE_END..OPCODE_END + 7]);

                    if body_flag == E::ZERO || funct3_flag == E::ZERO || op_flag == E::ZERO {
                        return TOT_CNT * #n_constraints;
                    }

                    debug_assert!(result.len() >= constraint_degrees().len(), "result array too small");
                    let simm = get_i_imm(&current);
                    let upper_imm = get_u_imm(&current);
                    let pc = current[PC];
                    let h0 = next[H_0];
                    let h1 = next[H_1];
                    let h2 = next[H_2];
                    let h3 = next[H_3];
                    let h4 = next[H_4];
                    let h5 = next[H_5];
                    let jal_offset = get_jal_offset(&current);

                    let rd = get_rd(&next);
                    let rs1 = get_rs1(&current);
                    let rs2 = get_rs2(&current);
                    let rd_zero = binary_flag(&[E::ZERO, E::ZERO, E::ZERO, E::ZERO, E::ZERO], &current[RD_END..RD_END + #REG_NUM_PO2], E::ONE);
                      
                    for shamt in 0..SHAMT_CNT {
                        let shamt_flag = shamt_flag(shamt as u8, &current[SHAMT_END..SHAMT_END + 5]);
                        let cumulative_flag = op_flag * body_flag * funct3_flag * shamt_flag * (E::ONE - rd_zero);
                        #(
                            result[index] = (#c_exprs) * cumulative_flag;
                            index += 1;
                        )*
                    }

                    // return the number of used constraint columns
                    TOT_CNT * #n_constraints
                }

                pub fn constraint_degrees() -> Vec<TransitionConstraintDegree> {
                    let mut degrees = Vec::with_capacity(TOT_CNT);
                    for _ in 0..TOT_CNT {
                        for deg in CONSTRAINT_DEGS.iter() {
                            degrees.push(TransitionConstraintDegree::new( OPCODE_FLAG_DEG + FUNCT3_FLAG_DEG + BODY_FLAG_DEG + SHAMT_DEG + 5 + deg));
                        }
                    }
                    degrees
                }

                fn get_i_imm<E: FieldElement>(trace: &[E]) -> E {
                    get_signed::<12, 12, _>(&trace[IMM_END..IMM_END + 12])
                }

                fn get_u_imm<E: FieldElement>(trace: &[E]) -> E {
                    get_signed::<32, 20, _>(&trace[UIMM_END..UIMM_END + 20])
                }

                fn get_jal_offset<E: FieldElement>(trace: &[E]) -> E {
                    jal_offset(&trace[JAL_OFFSET_END..JAL_OFFSET_END + 20])
                }

                fn get_rd<E: FieldElement>(trace: &[E]) -> E {
                    get_signed::<REG_BITS, REG_BITS, _>(&trace[RD_BITS_END..RD_BITS_END + REG_BITS])
                }

                fn get_rs1<E: FieldElement>(trace: &[E]) -> E {
                    get_signed::<REG_BITS, REG_BITS, _>(&trace[RS1_BITS_END..RS1_BITS_END + REG_BITS])
                }

                fn get_rs2<E: FieldElement>(trace: &[E]) -> E {
                    get_signed::<REG_BITS, REG_BITS, _>(&trace[RS2_BITS_END..RS2_BITS_END + REG_BITS])
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

              
                fn jal_offset<E: FieldElement>(offset: &[E]) -> E {
                    // format for jal offset is a bit wonky:
                    // [20, 10-1, 11, 19-12]
                    assert_eq!(offset.len(), 20, "requested jal offset with invalid length {}", offset.len());
                    let mut result = E::ZERO;
                    result -= offset[0] * E::from(1u32 << 20);
                    for i in 1..=10 {
                        result += offset[i] * E::from(1u32 << (11 - i));
                    }
                    result += offset[11] * E::from(1u32 << 11);
                    for i in 12..20 {
                        result += offset[i] * E::from(1u32 << (19 - i + 12));
                    }
                    result
                }

                fn get_signed<const N: usize, const LEN: usize, E: FieldElement>(op: &[E]) -> E {
                    let mut result = E::ZERO;
                    assert_eq!(op.len(), LEN, "requested upper immediate with invalid length {}", op.len());
                    result -= op[0] * E::from(1u32 << (N - 1));
                    for i in 1..LEN {
                        result += op[i] * E::from(1u32 << (N - i - 1));
                    }
                    result
                }
        }
    }.into()
}
