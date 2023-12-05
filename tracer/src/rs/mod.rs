use crate::ExecutionTrace;

use risc0_fields::field::{Elem, Field};
use risc0_zkp::core::ntt::{evaluate_ntt, expand, interpolate_ntt};

use crate::constraints::Transition;
// Reed solomon expansion rate
const RS_RATE: usize = 4;
const RS_RATE_BITS: usize = 2; // ceil(log2(RS_RATE))

pub struct ExpandedTrace<E>(Vec<Vec<E>>);

impl<E> ExpandedTrace<E>
where
    E: Copy,
{
    pub fn get_transition(&self, idx: usize) -> Transition<E> {
        // self.0[0][0] is just any value, it will be overwritten
        let mut trace_1 = [self.0[0][0]; 64];
        let mut trace_2 = [self.0[0][0]; 64];
        for (i, column) in self.0.iter().enumerate() {
            trace_1[i] = column[idx - 1];
            trace_2[i] = column[idx];
        }
        Transition::new(trace_1.into(), trace_2.into())
    }

    pub fn len(&self) -> usize {
        self.0[0].len()
    }
}

pub fn rs_expansion<F>(trace: ExecutionTrace<F::Elem>) -> ExpandedTrace<F::Elem>
where
    F: Field, // E: Elem + RootsOfUnity + Copy + Mul<Output = E> + Add<Output = E> + Sub<Output = E>,
{
    assert!(trace.len().is_power_of_two());
    let expansion_len = trace.len() * RS_RATE;

    let columns = trace.into_trace_columns();
    let mut res = vec![vec![F::Elem::ONE; expansion_len]; columns.len()];
    // TODO: parallelize
    for (mut column, out) in columns.into_iter().zip(res.iter_mut()) {
        interpolate_ntt(&mut column);
        expand(out, &column, RS_RATE_BITS);
        evaluate_ntt(out, RS_RATE_BITS);
    }
    ExpandedTrace(res)
}

mod tests {

    #[test]
    fn test_rs_expansion() {
        let trace = (0u32..8).map(BabyBearElem::from).collect::<Vec<_>>();
        let expansion = rs_expansion(&trace);
        for i in 0..trace.len() {
            assert_eq!(expansion[i * RS_RATE], trace[i]);
        }
        println!("trace: {:?}", trace);
        println!("expansion: {:?}", expansion);
    }
}
