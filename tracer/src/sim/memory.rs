use std::{collections::BTreeMap, sync::Mutex};

use miden_air::trace::CHIPLETS_WIDTH;
use miden_core::Felt;
use miden_processor::{
    chiplets::{aux_trace::AuxTraceBuilder, Chiplets},
    math::StarkField,
};

use rvsim::MemoryAccess;
const SYS_CTX: u32 = 0x0;

static TRACE: Mutex<Option<Chiplets>> = Mutex::new(None);

pub struct Memory {
    clk: u32,
    // TODO: the memory api of rvsim are a mess, especially to interoperate with miden.
    // We will implement the memory locally and forward all accesses at the end to the miden backend.
    buf: BTreeMap<u32, u32>,
    regs: [u32; 32],
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
            regs: [0; 32],
        }
    }

    // Registers use a separate memory space
    pub fn get_reg(&mut self, reg: usize) -> u32 {
        assert!(reg < 32);
        // we probably want to handle r0 differently
        if reg == 0 {
            return 0;
        }
        self.regs[reg]
    }

    // Registers use a separate memory space
    pub fn save_reg(&mut self, reg: usize, val: u32) {
        assert!(reg < 32);
        // we probably want to handle r0 differently
        if reg == 0 {
            return;
        }
        self.regs[reg] = val;
    }

    pub fn clock(&self) -> u32 {
        self.clk
    }

    // increment the clock
    pub fn advance(&mut self) {
        self.clk += 1;
    }

    // access memory without enforcing constraints
    pub(super) fn get(&self, addr: u32) -> u32 {
        self.buf.get(&addr).copied().unwrap_or(0)
    }

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
        println!("{:?}", self.accesses);
        for access in self.accesses.iter().copied() {
            match access {
                Access::Load { addr, data, clk } => {
                    assert!(chiplet_clk <= clk);
                    while chiplet_clk < clk {
                        chiplets.advance_clock();
                        chiplet_clk += 1;
                    }
                    // chiplets.read_mem(SYS_CTX, addr);
                    assert_eq!(chiplets.read_mem(SYS_CTX, addr)[0].as_int() as u32, data);
                }
                Access::Store { addr, data, clk } => {
                    assert!(chiplet_clk <= clk);
                    while chiplet_clk < clk {
                        chiplets.advance_clock();
                        chiplet_clk += 1;
                    }
                    // chiplets.read_mem(SYS_CTX, addr);
                    chiplets.write_mem_element(SYS_CTX, addr, data.into());
                }
            }
        }
        chiplets
    }

    pub fn to_trace(
        &self,
        trace_len: usize,
        num_rand_rows: usize,
    ) -> ([Vec<Felt>; CHIPLETS_WIDTH], AuxTraceBuilder) {
        println!("building real trace");
        let mut trace = self
            .build_chiplet_trace()
            .into_trace(trace_len, num_rand_rows);

        for col in trace.trace.iter_mut() {
            // use rand::Rng;
            // let mut bytes = [0u8; 8];
            // rand::thread_rng().fill(&mut bytes[..]);
            // *col.last_mut().unwrap() = Felt::from(bytes);
        }
        (trace.trace, trace.aux_builder)
    }

    pub fn trace_len(&self) -> usize {
        self.build_chiplet_trace().trace_len()
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
                let data = self.buf.get(&addr).copied().unwrap_or(0);
                self.accesses.push(Access::Load {
                    addr,
                    data: data,
                    clk: self.clk,
                });
                unsafe { *ptr = *(&data as *const u32 as *const T) };
            }
            MemoryAccess::Store(val) => {
                let mut data: u32 = 0;
                unsafe { *(&mut data as *mut u32 as *mut T) = val };
                self.accesses.push(Access::Store {
                    addr,
                    data,
                    clk: self.clk,
                });
                self.buf.insert(addr, data);
            }
        }
        // FIXME: to avoid accessing the same memory location twice at the same cycle, let's simply advance the clock
        // after each access. However, this makes it almost hard to track the cycle count.
        // self.advance();
        true
    }
}
