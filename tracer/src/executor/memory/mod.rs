/// An implementation of memory constraints for a STARK proof
///
/// Largely taken from https://github.com/0xPolygonMiden/miden-vm
use miden_air::trace::CHIPLETS_WIDTH;
use miden_processor::{
    chiplets::{aux_trace::AuxTraceBuilder, Chiplets},
    math::StarkField,
};
use rvsim::MemoryAccess;
use std::collections::BTreeMap;
use winterfell::math::fields::f64::BaseElement;

const SYS_CTX: u32 = 0x0;
pub const MEMORY_TRACE_WIDTH: usize = CHIPLETS_WIDTH;
const NUM_RAND_ROWS: usize = 1;

pub struct Memory {
    clk: u32,
    // TODO: the memory api of rvsim are a mess, especially to interoperate with miden.
    // We will implement the memory locally and forward all accesses at the end to the miden backend.
    buf: BTreeMap<u32, u32>,
    accesses: Vec<Access>,
}

#[derive(Clone, Copy, Debug)]
enum Access {
    Load { addr: u32, data: u32, clk: u32 },
    Store { addr: u32, data: u32, clk: u32 },
}

impl Memory {
    pub fn new() -> Self {
        Self {
            clk: 0,
            buf: BTreeMap::new(),
            accesses: Vec::new(),
        }
    }

    pub fn clock(&self) -> u32 {
        self.clk
    }

    /// Increment the internal clock
    pub fn advance(&mut self) {
        self.clk += 1;
    }

    /// access memory without enforcing constraints
    pub fn get(&self, addr: u32) -> u32 {
        self.buf.get(&addr).copied().unwrap_or(0)
    }

    /// Read a value from memory
    pub fn load(&mut self, addr: u32) -> u32 {
        let data = self.buf.get(&addr).copied().unwrap_or(0);
        self.accesses.push(Access::Load {
            addr,
            data,
            clk: self.clk,
        });
        data
    }

    /// Store a value into memory
    pub fn store(&mut self, addr: u32, data: u32) {
        self.buf.insert(addr, data);
        self.accesses.push(Access::Store {
            addr,
            data,
            clk: self.clk,
        });
    }

    fn build_chiplet_trace(&self) -> Chiplets {
        // We're only going to use the memory (and possibly range checker), but this
        // wrapper makes it convenient to use it.
        // TODO: extract only the parts that we need
        let mut chiplets = Chiplets::new(Default::default());
        let mut chiplet_clk = 0;
        for access in self.accesses.iter().copied() {
            match access {
                Access::Load { addr, data, clk } => {
                    assert!(chiplet_clk <= clk);
                    while chiplet_clk < clk {
                        chiplets.advance_clock();
                        chiplet_clk += 1;
                    }
                    assert_eq!(chiplets.read_mem(SYS_CTX, addr)[0].as_int() as u32, data);
                }
                Access::Store { addr, data, clk } => {
                    assert!(chiplet_clk <= clk);
                    while chiplet_clk < clk {
                        chiplets.advance_clock();
                        chiplet_clk += 1;
                    }
                    chiplets.write_mem_element(SYS_CTX, addr, data.into());
                }
            }
        }
        chiplets
    }

    /// Generate a trace for the memory accesses
    ///
    /// The layout of the memory access trace is shown below.
    ///
    ///   s0   s1     0   addr   clk   v0    0    0    0   d0   d1   d_inv
    /// ├────┴────┴─────┴──────┴─────┴────┴────┴────┴────┴────┴────┴───────┤
    ///
    /// In the above, the meaning of the columns is as follows:
    /// - `s0` is a selector column used to identify whether the memory access is a read or a write. A
    ///   value of ZERO indicates a write, and ONE indicates a read.
    /// - `s1` is a selector column used to identify whether the memory access is a read of an existing
    ///   memory value or not (i.e., this context/addr combination already existed and is being read).
    ///   A value of ONE indicates a read of existing memory, meaning the previous value must be copied.
    /// - `addr` contains memory address. Values in this column must increase monotonically for a
    ///   given context but there can be gaps between two consecutive values of up to 2^32. Also,
    ///   two consecutive values can be the same.
    /// - `clk` contains clock cycle at which a memory operation happened. Values in this column must
    ///   increase monotonically for a given context and memory address but there can be gaps between
    ///   two consecutive values of up to 2^32.
    /// - Column `v0` contain the field element stored at a given address/clock cycle after the memory operation.
    /// - Columns `d0` and `d1` contain lower and upper 16 bits of the delta between two consecutive
    ///   addresses or clock cycles. Specifically:
    ///   - When the address changes, these columns contain
    ///     (`new_addr` - `old-addr`).
    ///   - When both the context and the address remain the same, these columns contain
    ///     (`new_clk` - `old_clk` - 1).
    /// - `d_inv` contains the inverse of the delta between two consecutive addresses or
    ///   clock cycles computed as described above.
    ///
    /// For the first row of the trace, values in `d0`, `d1`, and `d_inv` are set to zeros.
    pub fn into_trace(
        self,
        trace_len: usize,
    ) -> ([Vec<BaseElement>; MEMORY_TRACE_WIDTH], AuxTraceBuilder) {
        assert!(self.trace_len() <= trace_len);
        let trace = self
            .build_chiplet_trace()
            .into_trace(trace_len, NUM_RAND_ROWS);

        (trace.trace, trace.aux_builder)
    }

    pub fn trace_len(&self) -> usize {
        self.build_chiplet_trace().trace_len()
    }
}

impl Default for Memory {
    fn default() -> Self {
        Self::new()
    }
}

impl rvsim::Memory for Memory {
    fn access<T: Copy>(&mut self, addr: u32, access: MemoryAccess<T>) -> bool {
        // TODO: not a fan of this api and this hack
        let size = std::mem::size_of::<T>();
        assert!(size <= 4);
        assert!(addr % 4 == 0, "unaligned access: {} {}", addr, addr % 4);

        match access {
            MemoryAccess::Load(ptr) | MemoryAccess::Exec(ptr) => {
                let data = self.load(addr);
                unsafe { *ptr = *(&data as *const u32 as *const T) };
            }
            MemoryAccess::Store(val) => {
                let mut data: u32 = 0;
                unsafe { *(&mut data as *mut u32 as *mut T) = val };
                self.store(addr, data);
            }
        }
        true
    }
}
