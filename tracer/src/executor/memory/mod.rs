/// An implementation of memory constraints for a STARK proof
///
/// Largely taken from https://github.com/0xPolygonMiden/miden-vm
use miden_air::trace::CHIPLETS_WIDTH;
use miden_processor::{
    chiplets::{aux_trace::AuxTraceBuilder, Chiplets},
    math::StarkField,
    range::RangeChecker,
};
use rvsim::MemoryAccess;
use std::collections::BTreeMap;
use winterfell::math::fields::f64::BaseElement;

const MEM_CTX: u32 = 0x0;
const REG_CTX: u32 = 0x1;
pub const MEMORY_TRACE_WIDTH: usize = CHIPLETS_WIDTH;
const NUM_RAND_ROWS: usize = 1;

// Both for the register file and the memory, we need to support up to 3 accesses to the same region of memory
// in the same clock cycle.
// However, the current memory implementation only allows to access each address once per clock cycle.
// To overcome this limitation, we will overclock the memory by a factor of 4, thus allowing 4 accesses per cycle.
// This does not change the efficiency or semantics of the memory, but requires a bit more consideration when building
// the bus trace.
// In particular, all requests will appear in the bus at the original cycle, but lookups will be calculated using
// the 'overclocked' cycle as specified below for each access.
const OVERCLOCK: u32 = 4;

/// The register file is implemented using a special region in memory, and thus
/// reuse the same constrainst.
/// This implementation assumes exactly 2 reads and 1 store are performed at each cycle.
/// Clocks for lookups are then calculated as follows:
/// rs1: bus_clock * OVERCLOCK
/// rs2: bus_clock * OVERCLOCK + 1
/// rd : bus_clock * OVERCLOCK + 2
pub struct RegisterFile {
    regs: [u32; 32],
    accesses: Vec<Access>,
    clk: u32,
    loads: u32,
    stores: u32,
}

impl Default for RegisterFile {
    fn default() -> Self {
        Self::new()
    }
}

impl RegisterFile {
    pub fn new() -> Self {
        Self {
            regs: [0; 32],
            accesses: Vec::new(),
            clk: 0,
            loads: 0,
            stores: 0,
        }
    }

    pub fn advance_clk(&mut self) {
        self.clk += 1;
        self.loads = 0;
        self.stores = 0;
    }

    // Load registers value, this is expected to happen at the end of the clock cycle
    pub fn load(&mut self, reg: u32) -> u32 {
        assert_eq!(self.stores, 0, "Loading after a store in the same cycle");
        assert!(self.loads < 2, "Only 2 loads are supported for each cycle");
        self.accesses.push(Access::Load {
            addr: reg,
            mem_clk: self.clk * OVERCLOCK + self.loads,
            data: self.regs[reg as usize],
            ctx: REG_CTX,
        });
        self.loads += 1;
        self.regs[reg as usize]
    }

    // Store register value, this is expected to happen at the start of the clock cycle
    pub fn store(&mut self, reg: u32, val: u32) {
        assert_eq!(self.stores, 0, "Already performed a store in this cycle");
        assert_eq!(self.loads, 2, "Storing before 2 loeads in the same cycle",);
        self.accesses.push(Access::Store {
            addr: reg,
            mem_clk: self.clk * OVERCLOCK + 2,
            data: val,
            ctx: REG_CTX,
        });
        self.stores += 1;
        self.regs[reg as usize] = val;
    }
}

pub struct Memory {
    register_file: RegisterFile,
    bus_clk: u32,
    // TODO: the memory api of rvsim are a mess, especially to interoperate with miden.
    // We will implement the memory locally and forward all accesses at the end to the miden backend.
    buf: BTreeMap<u32, u32>,
    accesses: Vec<Access>,
    chiplets_trace: Option<Chiplets>,
}

#[derive(Clone, Copy, Debug)]
enum Access {
    Load {
        addr: u32,
        data: u32,
        mem_clk: u32,
        ctx: u32,
    },
    Store {
        addr: u32,
        data: u32,
        mem_clk: u32,
        ctx: u32,
    },
}

impl Access {
    fn mem_clk(&self) -> u32 {
        match self {
            Access::Load { mem_clk, .. } | Access::Store { mem_clk, .. } => *mem_clk,
        }
    }
}

impl Memory {
    pub fn new() -> Self {
        Self {
            bus_clk: 0,
            buf: BTreeMap::new(),
            accesses: Vec::new(),
            register_file: RegisterFile::new(),
            chiplets_trace: None,
        }
    }

    pub fn bus_clock(&self) -> u32 {
        self.bus_clk
    }

    /// Increment the cpu clock
    pub fn advance_bus_clk(&mut self) {
        self.bus_clk += 1;
        self.register_file.advance_clk();
    }

    /// access memory without enforcing constraints
    pub fn get(&self, addr: u32) -> u32 {
        self.buf.get(&addr).copied().unwrap_or(0)
    }

    pub fn register_file(&mut self) -> &mut RegisterFile {
        &mut self.register_file
    }

    /// Read a value from memory
    pub fn load(&mut self, addr: u32) -> u32 {
        let data = self.buf.get(&addr).copied().unwrap_or(0);
        self.accesses.push(Access::Load {
            addr,
            data,
            mem_clk: self.bus_clk * OVERCLOCK,
            ctx: MEM_CTX,
        });
        data
    }

    /// Store a value into memory
    pub fn store(&mut self, addr: u32, data: u32) {
        self.buf.insert(addr, data);
        self.accesses.push(Access::Store {
            addr,
            data,
            mem_clk: self.bus_clk * OVERCLOCK,
            ctx: MEM_CTX,
        });
    }

    fn build_chiplet_trace(&mut self) {
        debug_assert!(self.chiplets_trace.is_none());
        // We're only going to use the memory (and possibly range checker), but this
        // wrapper makes it convenient to use it.
        // TODO: extract only the parts that we need
        let mut chiplets = Chiplets::new(Default::default());
        let mut accesses = core::mem::take(&mut self.accesses);
        accesses.extend(self.register_file.accesses.drain(..));
        accesses.sort_by_key(|a| a.mem_clk());
        let mut chiplet_clk = 0;
        for access in accesses {
            match access {
                Access::Load {
                    addr,
                    data,
                    mem_clk,
                    ctx,
                } => {
                    let bus_clk = mem_clk / OVERCLOCK;
                    assert!(chiplet_clk <= mem_clk);
                    while chiplet_clk < mem_clk {
                        chiplets.advance_clock();
                        chiplet_clk += 1;
                    }

                    assert_eq!(
                        chiplets.read_mem(ctx, addr, bus_clk)[0].as_int() as u32,
                        data
                    );
                }
                Access::Store {
                    addr,
                    data,
                    mem_clk,
                    ctx,
                } => {
                    let bus_clk = mem_clk / OVERCLOCK;
                    assert!(chiplet_clk <= mem_clk);
                    while chiplet_clk < mem_clk {
                        chiplets.advance_clock();
                        chiplet_clk += 1;
                    }
                    chiplets.write_mem_element(ctx, addr, data.into(), bus_clk);
                }
            }
        }

        self.chiplets_trace = Some(chiplets);
    }

    pub fn append_range_checks(&mut self, range: &mut RangeChecker) {
        self.build_chiplet_trace();
        self.chiplets_trace
            .as_ref()
            .unwrap()
            .append_range_checks(range);
    }

    /// Generate a trace for the memory accesses
    ///
    /// The layout of the memory access trace is shown below.
    ///
    ///   s0   s1    ctx   addr   clk   v0    0    0    0   d0   d1   d_inv
    /// ├────┴────┴──────┴──────┴─────┴────┴────┴────┴────┴────┴────┴───────┤
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
        mut self,
        trace_len: usize,
    ) -> ([Vec<BaseElement>; MEMORY_TRACE_WIDTH], AuxTraceBuilder) {
        assert!(self.trace_len() <= trace_len);
        let trace = self
            .chiplets_trace
            .take()
            .unwrap()
            .into_trace(trace_len, NUM_RAND_ROWS);

        (trace.trace, trace.aux_builder)
    }

    pub fn trace_len(&self) -> usize {
        self.chiplets_trace.as_ref().unwrap().trace_len()
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
