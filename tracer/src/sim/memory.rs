/// A simple `Memory` implementation, that creates an address space with just some DRAM.
#[derive(Debug)]
pub struct SimpleMemory {
    pub dram: Vec<u8>,
}

impl SimpleMemory {
    pub const DRAM_BASE: u32 = 0x2000_0000;
    pub const DRAM_SIZE: usize = 0x100;

    pub fn new() -> Self {
        Self {
            dram: vec![0; Self::DRAM_SIZE],
        }
    }

    pub fn load_slice(&mut self, addr: u32, data: &[u8]) {
        let internal = addr - Self::DRAM_BASE;
        self.dram[internal as usize..internal as usize + data.len()].copy_from_slice(data);
    }
}

impl Default for SimpleMemory {
    fn default() -> Self {
        Self::new()
    }
}

/// Our implementation of `Memory` builds a simple memory map.
///
/// The `Memory` trait is also implemented for `[u8]`, so we can simply delegate to it, after
/// translating the address.
///
/// The condition here only checks the start address of DRAM, because the upper bound is
/// already checked by the `[u8]` implementation. This type of memory map can be easily
/// extended by adding more `else if` clauses, working through blocks of memory from highest
/// base address to lowest.
impl rvsim::Memory for SimpleMemory {
    fn access<T: Copy>(&mut self, addr: u32, access: rvsim::MemoryAccess<T>) -> bool {
        if addr >= Self::DRAM_BASE {
            rvsim::Memory::access(&mut self.dram[..], addr - Self::DRAM_BASE, access)
        } else {
            false
        }
    }
}
