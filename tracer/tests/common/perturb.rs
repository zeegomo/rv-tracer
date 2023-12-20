use super::*;
use prop::test_runner::TestRng;
use proptest::prelude::*;
use trace_defs::*;
use winterfell::math::StarkField;

// bits in a register
const REG_BITS: usize = 32;
// how many bits to identify a register
const REG_NUM_BITS: usize = 5;

#[derive(Debug, Clone)]
pub struct Rd;
#[derive(Debug, Clone)]
pub struct Rs1;
#[derive(Debug, Clone)]
pub struct Rs2;
#[derive(Debug, Clone)]
pub struct Imm;
#[derive(Debug, Clone)]
pub struct Uimm;
#[derive(Debug, Clone)]
pub struct RdBits;
#[derive(Debug, Clone)]
pub struct Rs1Bits;
#[derive(Debug, Clone)]
pub struct Rs2Bits;
#[derive(Debug, Clone)]
pub struct Pc;

pub trait Field: Debug {
    fn perturb<E: StarkField>(
        prev: &mut [E; TRACE_WIDTH],
        next: &mut [E; TRACE_WIDTH],
        rng: &mut TestRng,
    );
}

impl Field for Rd {
    fn perturb<E: StarkField>(
        prev: &mut [E; TRACE_WIDTH],
        _next: &mut [E; TRACE_WIDTH],
        rng: &mut TestRng,
    ) {
        perturb_reg::<RD_END, _>(prev, rng);
    }
}

impl Field for Rs1 {
    fn perturb<E: StarkField>(
        prev: &mut [E; TRACE_WIDTH],
        _next: &mut [E; TRACE_WIDTH],
        rng: &mut TestRng,
    ) {
        perturb_reg::<RS1_END, _>(prev, rng);
    }
}

impl Field for Rs2 {
    fn perturb<E: StarkField>(
        prev: &mut [E; TRACE_WIDTH],
        _next: &mut [E; TRACE_WIDTH],
        rng: &mut TestRng,
    ) {
        perturb_reg::<RS2_END, _>(prev, rng);
    }
}

impl Field for Imm {
    fn perturb<E: StarkField>(
        prev: &mut [E; TRACE_WIDTH],
        _next: &mut [E; TRACE_WIDTH],
        rng: &mut TestRng,
    ) {
        perturb_bits::<12, IMM_END, _>(prev, rng);
    }
}

impl Field for Uimm {
    fn perturb<E: StarkField>(
        prev: &mut [E; TRACE_WIDTH],
        _next: &mut [E; TRACE_WIDTH],
        rng: &mut TestRng,
    ) {
        perturb_bits::<20, UIMM_END, _>(prev, rng);
    }
}

impl Field for RdBits {
    fn perturb<E: StarkField>(
        _prev: &mut [E; TRACE_WIDTH],
        next: &mut [E; TRACE_WIDTH],
        rng: &mut TestRng,
    ) {
        perturb_reg_bits::<RD_BITS_END, _>(next, rng);
    }
}

impl Field for Rs1Bits {
    fn perturb<E: StarkField>(
        _prev: &mut [E; TRACE_WIDTH],
        next: &mut [E; TRACE_WIDTH],
        rng: &mut TestRng,
    ) {
        perturb_reg_bits::<RS1_BITS_END, _>(next, rng);
    }
}

impl Field for Rs2Bits {
    fn perturb<E: StarkField>(
        _prev: &mut [E; TRACE_WIDTH],
        next: &mut [E; TRACE_WIDTH],
        rng: &mut TestRng,
    ) {
        perturb_reg_bits::<RS2_BITS_END, _>(next, rng);
    }
}

impl Field for Pc {
    fn perturb<E: StarkField>(
        prev: &mut [E; TRACE_WIDTH],
        _next: &mut [E; TRACE_WIDTH],
        rng: &mut TestRng,
    ) {
        perturb_field(&mut prev[PC], rng);
    }
}

fn perturb_bits<const N: usize, const OFFSET: usize, E: StarkField>(
    trace: &mut [E; TRACE_WIDTH],
    rng: &mut TestRng,
) {
    let change = to_binary::<N>(rng.gen_range(1..(1u64 << N)));
    for i in 0..N {
        if E::from(change[i]) == E::ONE {
            trace[OFFSET + i] = E::ONE - trace[OFFSET + i];
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

fn perturb_reg<const OFFSET: usize, E: StarkField>(
    trace: &mut [E; TRACE_WIDTH],
    rng: &mut TestRng,
) {
    perturb_bits::<REG_NUM_BITS, OFFSET, _>(trace, rng);
}

fn perturb_reg_bits<const OFFSET: usize, E: StarkField>(
    trace: &mut [E; TRACE_WIDTH],
    rng: &mut TestRng,
) {
    perturb_bits::<REG_BITS, OFFSET, _>(trace, rng);
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
                val |= (*bit as u64) << (N - i - 1);
            }
            assert_eq!(val, i, "{i} to binary: {:?}", binary);
        }
    }
}
