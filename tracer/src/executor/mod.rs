mod cpu;
pub mod memory;
mod program;

use miden_processor::{range::RangeChecker, RangeCheckTrace};
pub use program::Program;
use std::time::Instant;
use trace_defs::CYCLE;

use crate::{
    executor::{cpu::Cpu, memory::Memory},
    trace::{AuxTraceBuilder, TraceTable},
    Felem,
};

// for some reason the trace length must be at least 8
const MIN_LEN: usize = 8;
const NUM_RAND_ROWS: usize = 1;

pub fn exec(program: &Program) -> TraceTable<Felem> {
    let mut memory = Memory::new();
    let start = Instant::now();
    let mut range = RangeChecker::new();
    let cpu = Cpu::run(program, &mut memory);
    log::debug!(
        "program loaded and executed in {} cycles / {} ms",
        cpu.trace_len(),
        start.elapsed().as_millis()
    );
    memory.append_range_checks(&mut range);
    let cpu_trace_len = cpu.trace_len();
    assert!(
        cpu_trace_len > 1,
        "the trace length was {cpu_trace_len}, maybe something went wrong?",
    );
    let memory_trace_len = memory.trace_len();
    let range_trace_len = range.get_number_range_checker_rows();
    let trace_len = cpu_trace_len
        .max(memory_trace_len)
        .max(range_trace_len)
        .next_power_of_two();
    let trace_len = core::cmp::max(trace_len, MIN_LEN);

    let cpu_trace = cpu.into_trace(trace_len);
    let (mem_trace, mem_aux_builder) = memory.into_trace(trace_len);

    let RangeCheckTrace {
        trace: mut range_check_trace,
        aux_builder: range_aux_builder,
    } = range.into_trace_with_table(range_trace_len, trace_len, NUM_RAND_ROWS);

    use miden_processor::crypto::RandomCoin;
    use miden_processor::crypto::RpoRandomCoin;
    let mut rng = RpoRandomCoin::new(&[(1u32.into())]);

    // inject random values into the last rows of the trace
    for i in trace_len - NUM_RAND_ROWS..trace_len {
        for column in &mut range_check_trace {
            column[i] = rng.draw().expect("failed to draw a random value");
        }
    }

    let trace = cpu_trace
        .into_iter()
        .chain(mem_trace)
        .chain(range_check_trace)
        .collect::<Vec<_>>();

    TraceTable::new(
        trace,
        AuxTraceBuilder::new(mem_aux_builder, range_aux_builder),
    )
}
