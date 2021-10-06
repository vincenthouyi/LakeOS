#![no_std]

use elf_rs::{ElfFile, ProgramHeaderWrapper, ProgramType};
use core::iter::Iterator;

pub trait ElfLoader {
    fn allocate<'a>(&mut self, load_headers: &mut dyn Iterator<Item=ProgramHeaderWrapper>) -> Result<(), &'static str>;
    fn load(&mut self, program_header: ProgramHeaderWrapper) -> Result<(), &'static str>;
    // fn relocate(&mut self, _entry: &Rela<P64>) -> Result<(), &'static str>
    fn load_elf(&mut self, elf: &dyn ElfFile) -> Result<(), &'static str> {
        let mut loadable_headers = elf.program_header_iter().filter(|ph| ph.ph_type() == ProgramType::LOAD);
        self.allocate(&mut loadable_headers)?;

        let loadable_headers = elf.program_header_iter().filter(|ph| ph.ph_type() == ProgramType::LOAD);
        for ph in loadable_headers {
            self.load(ph)?;
        }
        Ok(())
    }
}
