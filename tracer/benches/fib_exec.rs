use dhat::HeapStats;
use rv_tracer::executor::{exec, Program};
use rvsim::elf::Elf32;

#[global_allocator]
static ALLOC: dhat::Alloc = dhat::Alloc;

const FIBONACCI_ELF: &[u8] = include_bytes!("../../examples/fibonacci/fibonacci.bin");

fn fibonacci_1000() -> Program {
    let elf = Elf32::parse(FIBONACCI_ELF).unwrap();
    Program::from(&elf)
}

fn main() {
    let _profiler = dhat::Profiler::new_heap();
    exec(&fibonacci_1000());
    let HeapStats {
        total_blocks,
        total_bytes,
        max_blocks,
        max_bytes,
        ..
    } = dhat::HeapStats::get();
    println!(
        "
        Fibonacci 1000 exec stats:

        Total blocks: {total_blocks}
        Total bytes: {total_bytes}
        Max blocks: {max_blocks}
        Max bytes: {max_bytes}
    "
    );
}
