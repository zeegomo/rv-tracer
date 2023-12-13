use prop::test_runner::TestRng;
use proptest::prelude::*;
use trace_defs::*;
use winterfell::math::StarkField;

pub struct Perturb;
const REG_BITS: usize = 5;

pub enum Field {
    Rd,
    Rs1,
    Rs2,
    Imm,
    Uimm,
}

impl Field {
    pub fn perturb<E: StarkField>(&self, trace: &mut [E; TRACE_WIDTH], rng: &mut TestRng) {
        match self {
            Field::Rd => {
                let change = to_binary::<REG_BITS>(rng.gen_range(1..32));
                for i in 0..REG_BITS {
                    if E::from(change[i]) == E::ONE {
                        trace[RS1_END + i] = E::ONE - trace[RS1_END + i];
                    }
                }
            }
            Field::Uimm => {
                let change = to_binary::<20>(rng.gen_range(1..(1u32 << 20)));
                for i in 0..20 {
                    if E::from(change[i]) == E::ONE {
                        trace[UIMM_END + i] = E::ONE - trace[UIMM_END + i];
                    }
                }
            }
            _ => todo!(),
        }
    }
}

// mod fields {
//     use super::*;
//     use proptest::prelude::*;

//     const REG_BITS: usize = 5;

//     #[derive(Debug)]
//     pub struct Rd;

//     impl Field for Rd {
//         fn perturb(trace: &mut [u32; TRACE_WIDTH], rng: &mut TestRng) {
//             let change = to_binary::<REG_BITS>(rng.gen_range(1..32));
//             for i in 0..REG_BITS {
//                 trace[RS1_BITS_END + i] = change[i];
//             }
//         }
//     }

//     fn to_binary<const M: usize>(reg: u8) -> [u32; M] {
//         let mut result = [0; M];
//         assert!(
//             reg < (1 << M),
//             "requested binary representation of value({reg}) bigger than output array({M})"
//         );
//         for i in 0..M {
//             if reg & (1 << i) != 0 {
//                 result[M - i - 1] = 1;
//             }
//         }

//         result
//     }
// }

fn to_binary<const M: usize>(val: u32) -> [u32; M] {
    let mut result = [0; M];
    assert!(
        val < (1 << M),
        "requested binary representation of value({val}) bigger than output array({M})"
    );
    for i in 0..M {
        if val & (1 << i) != 0 {
            result[M - i - 1] = 1;
        }
    }

    result
}
