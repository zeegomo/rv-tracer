use miden_processor::{
    range::{self, AuxTraceBuilder},
    RangeCheckTrace,
};
use trace_defs::RANGE_WIDTH;

use crate::Felem;

const NUM_RAND_ROWS: usize = 1;

pub struct RangeChecker {
    inner: range::RangeChecker,
}

impl RangeChecker {
    pub fn new() -> Self {
        Self {
            inner: range::RangeChecker::new(),
        }
    }

    pub fn get_number_range_checker_rows(&self) -> usize {
        self.inner.get_number_range_checker_rows()
    }

    pub fn get_mut(&mut self) -> &mut range::RangeChecker {
        &mut self.inner
    }

    pub fn into_trace(self, range_trace_len: usize, trace_len: usize) -> RangeCheckTrace {
        let mut trace = self
            .inner
            .into_trace_with_table(range_trace_len, trace_len, NUM_RAND_ROWS);

        use miden_processor::crypto::RandomCoin;
        use miden_processor::crypto::RpoRandomCoin;
        let mut rng = RpoRandomCoin::new(&[(1u32.into())]);

        // inject random values into the last rows of the trace
        for col in trace.trace.iter_mut() {
            *col.last_mut().unwrap() = rng.draw().expect("failed to draw a random value");
        }

        trace
    }

    #[allow(clippy::type_complexity)]
    pub fn into_trace_with_splits(
        self,
        range_trace_len: usize,
        n_segments: usize,
        segment_len: usize,
    ) -> ([Vec<Felem>; RANGE_WIDTH], AuxTraceBuilder) {
        assert!(segment_len.is_power_of_two());
        // the first segment can hold segment_len - 1 rows from the original execution (the last one in padding)
        // while successive segments can only hold segment_len - 2 rows from the original execution
        // (1 is used for padding and 1 is the last row of the previous segment)
        let trace_len = n_segments * (segment_len - 2) + 1;
        assert!(trace_len >= range_trace_len);

        let trace_len = trace_len.next_power_of_two();
        let RangeCheckTrace { trace, aux_builder } = self.into_trace(range_trace_len, trace_len);
        (trace, aux_builder)
    }
}
