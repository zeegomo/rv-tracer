use rvsim::elf::{self, Elf32};
#[cfg(feature = "integration-test")]
use rvsim::CpuState;

pub struct Program {
    entry: u32,
    segments: Vec<(u32, Vec<u8>)>,
    #[cfg(feature = "integration-test")]
    starting_state: CpuState,
}

impl Program {
    pub fn new(entry: u32, segments: Vec<(u32, Vec<u8>)>) -> Self {
        Self {
            entry,
            segments,
            #[cfg(feature = "integration-test")]
            starting_state: CpuState::new(entry),
        }
    }

    pub fn entrypoint(&self) -> u32 {
        self.entry
    }

    pub fn segments(&self) -> &[(u32, Vec<u8>)] {
        &self.segments
    }

    #[cfg(feature = "integration-test")]
    pub fn set_starting_state(&mut self, state: CpuState) {
        self.starting_state = state;
    }

    #[cfg(feature = "integration-test")]
    pub fn starting_state(&self) -> CpuState {
        self.starting_state.clone()
    }
}

impl<'a> From<&Elf32<'a>> for Program {
    fn from(elf: &Elf32<'a>) -> Self {
        if elf.ident.data != elf::ELF_IDENT_DATA_2LSB
            || elf.ident.abi != elf::ELF_IDENT_ABI_SYSV
            || elf.header.typ != elf::ELF_TYPE_EXECUTABLE
            || elf.header.machine != elf::ELF_MACHINE_RISCV
        {
            panic!("unsupported executable format");
        }

        let mut segments = Vec::new();
        for (i, ph) in elf.ph.iter().enumerate() {
            let addr = ph.vaddr;
            if ph.typ == rvsim::elf::ELF_PROGRAM_TYPE_LOADABLE {
                segments.push((addr, elf.p[i].to_vec()));
            }
        }

        Self {
            entry: elf.header.entry,
            segments,
            #[cfg(feature = "integration-test")]
            starting_state: CpuState::new(elf.header.entry),
        }
    }
}
