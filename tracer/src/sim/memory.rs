use std::collections::BTreeMap;

/// A simple `Memory` implementation, that creates an address space with just some DRAM.
#[derive(Debug)]
pub struct SimpleMemory {
    pub dram: BTreeMap<u64, u8>,
}

impl SimpleMemory {
    pub fn new() -> Self {
        Self {
            dram: Default::default(),
        }
    }

    pub fn load_slice(&mut self, addr: u32, data: &[u8]) {
        for (i, byte) in data.iter().enumerate() {
            self.dram.insert(addr as u64 + i as u64, *byte);
        }
    }
}

impl Default for SimpleMemory {
    fn default() -> Self {
        Self::new()
    }
}

impl rvsim::Memory for SimpleMemory {
    fn access<T: Copy>(&mut self, addr: u32, access: rvsim::MemoryAccess<T>) -> bool {
        // TODO: not a fan of this api and this hack
        let size = std::mem::size_of::<T>();
        let addr = addr as u64;
        let mut slice = (addr..addr + size as u64)
            .map(|addr| self.dram.get(&addr).copied().unwrap_or(0))
            .collect::<Vec<_>>();
        let res = slice.access(0, access);
        for (i, byte) in slice.into_iter().enumerate() {
            self.dram.insert(addr + i as u64, byte);
        }
        res
    }
}
