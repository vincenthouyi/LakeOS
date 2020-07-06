use crate::debug_println as kprintln;
use rustyl4api::object::{VTableCap, RamObj, RamCap};
use rustyl4api::vspace::Permission;
use hashbrown::HashMap;

#[derive(Debug)]
enum VTableEntry {
    Table(VTable),
    Page(Page)
}

#[derive(Debug, Clone)]
pub struct Page {
    cap: RamCap,
}

#[derive(Debug)]
pub struct VTable {
    cap: VTableCap,
    entries: HashMap<usize, VTableEntry>,
}

#[derive(Debug, Default)]
pub struct VSpace {
    table: Option<VTable>,
}

impl VSpace {
    pub fn map_frame(&mut self, frame: RamCap, vaddr: usize, perm: Permission) -> Result<(), ()> {
        unimplemented!()
    }
}

#[derive(Debug, Default)]
pub struct Child {
    vspace: VSpace,
}

pub fn spawn_process_from_elf(elf_file: &[u8]) -> Result<(), ()> {
    use elf_rs::{Elf, ProgramType};
    use rustyl4api::vspace::{FRAME_BIT_SIZE, FRAME_SIZE};
    use crate::space_manager::INIT_ALLOC;

    let elf = Elf::from_bytes(elf_file).map_err(|_| ())?;
    let mut child = Child::default();

    if let Elf::Elf64(e) = elf {
        for ph in e.program_header_iter()
        {
            kprintln!("{:?}", ph);
            let p_flags = ph.ph.flags();
            let perm = Permission::new (
                p_flags & 0b100 == 0b100,
                p_flags & 0b010 == 0b010,
                p_flags & 0b001 == 0b001,
            );
            let p_type = ph.ph.ph_type();

            match p_type {
                ProgramType::LOAD => {
                    let sec_base = ph.ph.offset() as usize;
                    let sec_len= ph.ph.filesz() as usize;
                    let section = &elf_file[sec_base.. sec_base + sec_len];

                    let vaddr = ph.ph.vaddr() as usize;
                    let mem_len = ph.ph.memsz() as usize;
                    let memrange = vaddr..vaddr+mem_len;

                    memrange.step_by(FRAME_SIZE)
                        .map(|vaddr| {
                            let frame_cap = INIT_ALLOC.alloc_object::<RamObj>(FRAME_BIT_SIZE).unwrap();

                            child.vspace.map_frame(frame_cap.clone(), vaddr, perm).unwrap();

                            let frame_addr = INIT_ALLOC.insert_ram(frame_cap.clone(), Permission::writable());
                            let frame = unsafe {
                                core::slice::from_raw_parts_mut(frame_addr, FRAME_SIZE)
                            };
                            frame
                        })
                        .zip(section.chunks(FRAME_SIZE).chain(core::iter::repeat(&[0u8; 4096][..])))
                        .for_each(|(mut frame, page)| {
                            frame[..page.len()].copy_from_slice(page);
                        })
                }
                ProgramType::GNU_STACK => {
                    for i in 0..512 {
                        let vaddr = 0x8000000 - FRAME_SIZE * (i + 1);
                        let frame_cap = INIT_ALLOC.alloc_object::<RamObj>(FRAME_BIT_SIZE).unwrap();

                        child.vspace.map_frame(frame_cap.clone(), vaddr, perm).unwrap();

                        let frame_addr = INIT_ALLOC.insert_ram(frame_cap.clone(), Permission::writable());
                        let frame = unsafe {
                            core::slice::from_raw_parts_mut(frame_addr, FRAME_SIZE)
                        };

                        for b in frame {
                            *b = 0;
                        }
                    }
                }
                p_type => {
                    panic!("Unable to handle section type {:?}", p_type);
                }
            }
        }
//
//        let start_addr = e.header().entry_point() as usize;
//        tcb.tf.set_elr(start_addr);
//        tcb.tf.set_sp(INIT_STACK_TOP);
//        tcb.tf.set_spsr(0b1101 << 6 | 0 << 4 | 0b00 << 2 | 0b0 << 0); // set DAIF, AArch64, EL0t
    } else {
        unimplemented!("Elf32 binary is not supported!");
    }


    kprintln!("here");

    Ok(())
}