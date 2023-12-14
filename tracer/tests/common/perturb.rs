use super::*;
use prop::test_runner::TestRng;
use proptest::prelude::*;
use trace_defs::*;
use winterfell::math::StarkField;

// bits in a register
const REG_BITS: usize = 32;
// how many bits to identify a register
const REG_NUM_BITS: usize = 5;

pub enum Field {
    Rd,
    Rs1,
    Rs2,
    Imm,
    Uimm,
    RdBits,
    Rs1Bits,
    Rs2Bits,
    Pc,
}

impl Field {
    pub fn perturb<E: StarkField>(&self, trace: &mut [E; TRACE_WIDTH], rng: &mut TestRng) {
        match self {
            Field::Rd => {
                perturb_reg(trace, rng, RD_END);
            }
            Field::Rs1 => {
                perturb_reg(trace, rng, RS1_END);
            }
            Field::Rs2 => {
                perturb_reg(trace, rng, RS2_END);
            }
            Field::Uimm => {
                let change = to_binary::<20>(rng.gen_range(1..(1u32 << 20)));
                for i in 0..20 {
                    if E::from(change[i]) == E::ONE {
                        trace[UIMM_END + i] = E::ONE - trace[UIMM_END + i];
                    }
                }
            }
            Field::Pc => perturb_field(&mut trace[PC], rng),
            Field::Rs1Bits => {
                perturb_reg_bits(trace, rng, RS1_BITS_END);
            }
            Field::Rs2Bits => {
                perturb_reg_bits(trace, rng, RS2_BITS_END);
            }
            Field::RdBits => {
                perturb_reg_bits(trace, rng, RD_BITS_END);
            }
            _ => todo!(),
        }
    }
}

fn perturb_field<E: StarkField>(field: &mut E, rng: &mut TestRng) {
    let mut new = E::from(rng.gen_range(0..=u32::MAX));
    while new == *field {
        new = E::from(rng.gen_range(0..=u32::MAX));
    }
    *field = new;
}

fn perturb_reg<E: StarkField>(trace: &mut [E; TRACE_WIDTH], rng: &mut TestRng, offset: usize) {
    let change = to_binary::<REG_NUM_BITS>(rng.gen_range(1..(1u32 << REG_NUM_BITS)));
    for i in 0..REG_NUM_BITS {
        if E::from(change[i]) == E::ONE {
            trace[offset + i] = E::ONE - trace[offset + i];
        }
    }
}

fn perturb_reg_bits<E: StarkField>(trace: &mut [E; TRACE_WIDTH], rng: &mut TestRng, offset: usize) {
    let change = to_binary::<REG_BITS>(rng.gen_range(1..(1u64 << REG_BITS)) as u32);
    for i in 0..REG_BITS {
        if E::from(change[i]) == E::ONE {
            trace[offset + i] = E::ONE - trace[offset + i];
        }
    }
}

#[cfg(test)]
mod test {
    #[test]
    fn test_to_binary() {
        const N: usize = 10;
        for i in 0..(1 << N) {
            let binary = super::to_binary::<N>(i);
            let mut val = 0;
            for (i, bit) in binary.iter().enumerate() {
                val |= bit << (N - i - 1);
            }
            assert_eq!(val, i, "{i} to binary: {:?}", binary);
        }
    }
}
