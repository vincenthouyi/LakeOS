#![no_std]

use core::ops::Range;

use elf_rs::{
    Elf,
    ProgramType,
};

pub fn load_elf<'a>(elf_file: &[u8], stack_top: u64, stack_size: usize, frame_alloc_fn: &mut dyn FnMut(Range<u64>, u32) -> &'a mut [u8])
    -> Result<u64, &'static str>
{

    match Elf::from_bytes(elf_file).unwrap() {
        Elf::Elf64(e) => {
            let entry = e.header().entry_point() as usize;

            for ph in e.program_header_iter() {
                let flags = ph.ph.flags();

                match ph.ph.ph_type() {
                    ProgramType::LOAD => {
                        let align = ph.ph.align() as usize;
                        let seg_off = ph.ph.offset() as usize;
                        let seg_base = align_down(seg_off, align);
                        let seg_len= ph.ph.filesz() as usize;
                        let segment = &elf_file[seg_base.. seg_off + seg_len];

                        let vaddr = ph.ph.vaddr() as usize;
                        let vaddr_base = align_down(vaddr, align);
                        let mem_len = ph.ph.memsz() as usize;
                        let memrange = vaddr_base .. vaddr+mem_len;

                        memrange.step_by(4096)
                            .map(|vaddr| {
                                let vaddr = vaddr as u64;
                                frame_alloc_fn(vaddr .. vaddr + 4096, flags)
                            })
                            .zip(segment.chunks(4096).chain(core::iter::repeat(&[0u8; 4096][..])))
                            .for_each(|(frame, page)| {
                                frame[..page.len()].copy_from_slice(page);
                            })
                    }
                    ProgramType::GNU_STACK => {
                        let mut cur = stack_top - 4096;
                        while cur >= stack_top - stack_size as u64 {
                            let frame_range = frame_alloc_fn(cur .. cur + 4096, flags);
                            for b in &mut frame_range[..] { *b = 0 }
                            cur -= 4096;
                        }

                    }
                    _ => { }
                }
            }
            Ok(entry as u64)
        }
        Elf::Elf32(_) => { unimplemented!() }
    }
}

pub const fn align_down(addr: usize, align: usize) -> usize {
    addr & !(align - 1)
}