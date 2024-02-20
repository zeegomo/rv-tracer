mod cpu;
pub mod memory;
mod program;
mod range;
mod utils;

use miden_processor::RangeCheckTrace;
pub use program::Program;
use range::RangeChecker;
use std::{rc::Rc, time::Instant};
use winterfell::math::FieldElement;

use crate::{
    air::SegmentConfig,
    executor::{cpu::Cpu, memory::Memory},
    trace::{AuxTraceBuilder, TraceTable},
    Felem,
};

// for some reason the trace length must be at least 8
const MIN_LEN: usize = 8;

pub fn exec(program: &Program, segment_config: SegmentConfig) -> Vec<TraceTable<Felem>> {
    let mut memory = Memory::new();
    let start = Instant::now();
    let mut range = RangeChecker::new();
    let cpu = Cpu::run(program, &mut memory);
    log::debug!(
        "program loaded and executed in {} cycles / {} ms",
        cpu.trace_len(),
        start.elapsed().as_millis()
    );
    memory.append_range_checks(range.get_mut());
    let cpu_trace_len = cpu.trace_len();
    assert!(
        cpu_trace_len > 1,
        "the trace length was {cpu_trace_len}, maybe something went wrong?",
    );
    let memory_trace_len = memory.trace_len();
    let range_trace_len = range.get_number_range_checker_rows();

    let max_trace_len = cpu_trace_len.max(memory_trace_len).max(range_trace_len);

    match segment_config {
        SegmentConfig::Single => {
            let trace_len = max_trace_len.next_power_of_two();

            let trace_len = core::cmp::max(trace_len, MIN_LEN);

            let cpu_trace = cpu.into_trace(trace_len);
            let (mem_trace, mem_aux_builder) = memory.into_trace(trace_len);

            let RangeCheckTrace {
                trace: range_check_trace,
                aux_builder: range_aux_builder,
            } = range.into_trace(range_trace_len, trace_len);

            let trace = cpu_trace
                .into_iter()
                .chain(mem_trace)
                .chain(range_check_trace)
                .collect::<Vec<_>>();

            vec![TraceTable::new(
                trace,
                AuxTraceBuilder::new(mem_aux_builder, range_aux_builder),
            )]
        }
        SegmentConfig::Split { segment_len } => {
            assert!(segment_len.is_power_of_two());
            let remaining_trace_len = (max_trace_len as u32).saturating_sub(segment_len - 1);
            let available_segment_length = segment_len - 2;
            let n_segments = (remaining_trace_len + available_segment_length - 1)
                / (available_segment_length)
                + 1;

            let (cpu_traces, mut cpu_full_trace) =
                cpu.into_trace_with_splits(n_segments as usize, segment_len as usize);
            let (mem_traces, mut mem_full_trace, mem_aux_builder) =
                memory.into_trace_with_splits(n_segments as usize, segment_len as usize);

            let (range_traces, mut range_full_trace, aux_builder) = range.into_trace_with_splits(
                range_trace_len,
                n_segments as usize,
                segment_len as usize,
            );

            // FIXME: remove the assertion that a trace table must be a power of two
            assert!(
                cpu_full_trace[0].len() == mem_full_trace[0].len()
                    && mem_full_trace[0].len() == range_full_trace[0].len()
            );
            // we add padding rows so that the trace table will be a power of two, as required by winterfell
            // the added values will be ignored anyway
            let pad = cpu_full_trace[0].len().next_power_of_two() - cpu_full_trace[0].len();
            for col in &mut cpu_full_trace {
                col.extend(std::iter::repeat(Felem::ZERO).take(pad));
            }
            for col in &mut mem_full_trace {
                col.extend(std::iter::repeat(Felem::ZERO).take(pad));
            }
            for col in &mut range_full_trace {
                col.extend(std::iter::repeat(Felem::ZERO).take(pad));
            }

            let full_trace = Rc::new(winterfell::TraceTable::init(
                cpu_full_trace
                    .into_iter()
                    .chain(mem_full_trace)
                    .chain(range_full_trace)
                    .collect::<Vec<_>>(),
            ));

            let aux_builder = AuxTraceBuilder::new(mem_aux_builder, aux_builder);

            cpu_traces
                .into_iter()
                .zip(mem_traces)
                .zip(range_traces)
                .map(|((cpu, mem), range)| {
                    cpu.into_iter().chain(mem).chain(range).collect::<Vec<_>>()
                })
                .enumerate()
                .map(|(segment_n, columns)| {
                    TraceTable::new(
                        columns,
                        aux_builder.clone().segmented(
                            full_trace.clone(),
                            segment_n,
                            segment_len as usize,
                        ),
                    )
                })
                .collect::<Vec<_>>()
        }
    }
}
