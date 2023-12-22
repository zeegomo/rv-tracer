use std::num::NonZeroU64;

use super::*;
use proptest::prelude::*;
use quickcheck::{Arbitrary, Gen};
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
        rng: &mut Gen,
    );
}

impl Field for Rd {
    fn perturb<E: StarkField>(
        prev: &mut [E; TRACE_WIDTH],
        next: &mut [E; TRACE_WIDTH],
        rng: &mut Gen,
    ) {
        let rd = read_reg::<RD_END, _>(prev);
        perturb_field(&mut next[rd as usize], rng);
    }
}

impl Field for Rs1 {
    fn perturb<E: StarkField>(
        prev: &mut [E; TRACE_WIDTH],
        _next: &mut [E; TRACE_WIDTH],
        rng: &mut Gen,
    ) {
        let rs1 = read_reg::<RS1_END, _>(prev);
        perturb_field(&mut prev[rs1 as usize], rng);
    }
}

impl Field for Rs2 {
    fn perturb<E: StarkField>(
        prev: &mut [E; TRACE_WIDTH],
        _next: &mut [E; TRACE_WIDTH],
        rng: &mut Gen,
    ) {
        let rs2 = read_reg::<RS2_END, _>(prev);
        perturb_field(&mut prev[rs2 as usize], rng);
    }
}

impl Field for Imm {
    fn perturb<E: StarkField>(
        prev: &mut [E; TRACE_WIDTH],
        _next: &mut [E; TRACE_WIDTH],
        rng: &mut Gen,
    ) {
        perturb_bits::<12, IMM_END, _>(prev, rng);
    }
}

impl Field for Uimm {
    fn perturb<E: StarkField>(
        prev: &mut [E; TRACE_WIDTH],
        _next: &mut [E; TRACE_WIDTH],
        rng: &mut Gen,
    ) {
        perturb_bits::<20, UIMM_END, _>(prev, rng);
    }
}

impl Field for RdBits {
    fn perturb<E: StarkField>(
        _prev: &mut [E; TRACE_WIDTH],
        next: &mut [E; TRACE_WIDTH],
        rng: &mut Gen,
    ) {
        perturb_reg_bits::<RD_BITS_END, _>(next, rng);
    }
}

impl Field for Rs1Bits {
    fn perturb<E: StarkField>(
        _prev: &mut [E; TRACE_WIDTH],
        next: &mut [E; TRACE_WIDTH],
        rng: &mut Gen,
    ) {
        perturb_reg_bits::<RS1_BITS_END, _>(next, rng);
    }
}

impl Field for Rs2Bits {
    fn perturb<E: StarkField>(
        _prev: &mut [E; TRACE_WIDTH],
        next: &mut [E; TRACE_WIDTH],
        rng: &mut Gen,
    ) {
        perturb_reg_bits::<RS2_BITS_END, _>(next, rng);
    }
}

impl Field for Pc {
    fn perturb<E: StarkField>(
        prev: &mut [E; TRACE_WIDTH],
        _next: &mut [E; TRACE_WIDTH],
        rng: &mut Gen,
    ) {
        perturb_field(&mut prev[PC], rng);
    }
}

fn perturb_bits<const N: usize, const OFFSET: usize, E: StarkField>(
    trace: &mut [E; TRACE_WIDTH],
    rng: &mut Gen,
) {
    let mut change = u64::arbitrary(rng) & ((1 << N) - 1);
    while change == 0 {
        change = u64::arbitrary(rng) & ((1 << N) - 1);
    }
    // a 0 change would not be a perturbationd
    assert!(change != 0);
    let orig = get_signed::<N, N, _>(&trace[OFFSET..(OFFSET + N)]);
    let bin_change = to_binary::<N>(change.into());
    for i in 0..N {
        if E::from(bin_change[i]) == E::ONE {
            trace[OFFSET + i] = E::ONE - trace[OFFSET + i];
        }
    }
    let after = get_signed::<N, N, _>(&trace[OFFSET..(OFFSET + N)]);
    println!("change is {orig} -> {after} | {:?}", bin_change);
}

fn perturb_field<E: StarkField>(field: &mut E, rng: &mut Gen) {
    let mut new = E::from(u32::arbitrary(rng));
    while new == *field {
        new = E::from(u32::arbitrary(rng));
    }
    println!("change is {} -> {}", field, new);
    *field = new;
}

fn perturb_reg<const OFFSET: usize, E: StarkField>(trace: &mut [E; TRACE_WIDTH], rng: &mut Gen) {
    perturb_bits::<REG_NUM_BITS, OFFSET, _>(trace, rng);
}

fn perturb_reg_bits<const OFFSET: usize, E: StarkField>(
    trace: &mut [E; TRACE_WIDTH],
    rng: &mut Gen,
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

fn read_reg<const OFFSET: usize, E: StarkField>(trace: &[E; TRACE_WIDTH]) -> u32 {
    let mut val = 0;
    for i in 0..REG_NUM_BITS {
        if trace[OFFSET + i] == E::ONE {
            val |= 1 << (REG_NUM_BITS - i - 1);
        }
    }
    val
}

fn get_signed<const N: usize, const LEN: usize, E: StarkField>(op: &[E]) -> E {
    let mut result = E::ZERO;
    assert_eq!(
        op.len(),
        LEN,
        "requested upper immediate with invalid length {}",
        op.len()
    );
    result -= op[0] * E::from(1u32 << (N - 1));
    for i in 1..LEN {
        result += op[i] * E::from(1u32 << (N - i - 1));
    }
    result
}
