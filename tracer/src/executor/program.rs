use rvsim::elf::{self, Elf32};
use winterfell::math::{FieldElement, ToElements};

#[derive(Clone, Debug)]
pub struct Program {
    entry: u32,
    segments: Vec<(u32, Vec<u8>)>,
}

impl Program {
    pub fn new(entry: u32, segments: Vec<(u32, Vec<u8>)>) -> Self {
        Self { entry, segments }
    }

    pub fn entrypoint(&self) -> u32 {
        self.entry
    }

    pub fn segments(&self) -> &[(u32, Vec<u8>)] {
        &self.segments
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
        }
    }
}

impl<E: FieldElement> ToElements<E> for Program {
    fn to_elements(&self) -> Vec<E> {
        assert_eq!(
            self.segments.len(),
            1,
            "Only one segment is supported, got {}",
            self.segments.len()
        );
        let segment = &self.segments[0].1;
        // we only support non-compressed instructions which are 4 bytes each
        assert_eq!(segment.len() % 4, 0);
        let mut words = vec![E::from(self.entrypoint())];
        words.extend(
            segment
                .chunks_exact(4)
                .map(|bytes| u32::from_le_bytes(bytes.try_into().unwrap()))
                .map(E::from),
        );

        words
    }
}
