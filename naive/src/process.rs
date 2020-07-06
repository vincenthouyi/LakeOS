use rustyl4api::kprintln;
use rustyl4api::object::{VTableObj, RamObj, CNodeObj, TcbObj};
use rustyl4api::vspace::Permission;
use spaceman::vspace_man::{VSpaceMan, VSpaceManError};

#[derive(Debug)]
pub struct Child {
    vspace: VSpaceMan,
}

pub fn spawn_process_from_elf(elf_file: &[u8]) -> Result<(), ()> {
    use elf_rs::{Elf, ProgramType};
    use rustyl4api::vspace::{FRAME_BIT_SIZE, FRAME_SIZE};
    use crate::space_manager::gsm;

    let elf = Elf::from_bytes(elf_file).map_err(|_| ())?;

    let child_root_cn = gsm!().alloc_object::<CNodeObj>(16).unwrap();
    let child_root_vn = gsm!().alloc_object::<VTableObj>(12).unwrap();
    let mut child = Child {
        vspace: VSpaceMan::new(child_root_vn.clone()),
    };

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
                            use rustyl4api::object::RamCap;

                            let frame_cap = gsm!().alloc_object::<RamObj>(FRAME_BIT_SIZE).unwrap();
                            let frame_parent_slot = gsm!().cspace_alloc().unwrap();
                            frame_cap.derive(frame_parent_slot).unwrap();
                            let frame_parent_cap = RamCap::new(frame_parent_slot);

                            while let Err(e) = child.vspace.map_frame(frame_cap.clone(), vaddr, perm, 4) {
                                match e {
                                    VSpaceManError::SlotOccupied{level} => {
                                        panic!("slot occupied at level {} vaddr {:x}", level, vaddr);
                                    }
                                    VSpaceManError::SlotTypeError{level} => {
                                        panic!("wrong slot type at level {} vaddr {:x}", level, vaddr);
                                    }
                                    VSpaceManError::PageTableMiss{level} => {
                                        let vtable_cap = gsm!().alloc_object::<VTableObj>(12).unwrap();
                                        kprintln!("miss table level {} addr {:x}", level, vaddr);
                                        child.vspace.map_table(vtable_cap, vaddr, level).unwrap();
                                    }
                                }
                            };

                            let frame_addr = gsm!().insert_ram_at(frame_parent_cap.clone(), 0, Permission::writable());
                            let frame = unsafe {
                                core::slice::from_raw_parts_mut(frame_addr, FRAME_SIZE)
                            };
                            frame
                        })
                        .zip(section.chunks(FRAME_SIZE).chain(core::iter::repeat(&[0u8; 4096][..])))
                        .for_each(|(frame, page)| {
                            frame[..page.len()].copy_from_slice(page);
                        })
                }
                ProgramType::GNU_STACK => {
                    for i in 0..1 {
                        use rustyl4api::object::RamCap;

                        let vaddr = 0x8000000 - FRAME_SIZE * (i + 1);
                        let frame_cap = gsm!().alloc_object::<RamObj>(FRAME_BIT_SIZE).unwrap();
                        let frame_parent_slot = gsm!().cspace_alloc().unwrap();
                        frame_cap.derive(frame_parent_slot).unwrap();
                        let frame_parent_cap = RamCap::new(frame_parent_slot);

                        while let Err(e) = child.vspace.map_frame(frame_cap.clone(), vaddr, perm, 4) {
                            match e {
                                VSpaceManError::SlotOccupied{level} => {
                                    panic!("slot occupied at level {} vaddr {:x}", level, vaddr);
                                }
                                VSpaceManError::SlotTypeError{level} => {
                                    panic!("wrong slot type at level {} vaddr {:x}", level, vaddr);
                                }
                                VSpaceManError::PageTableMiss{level} => {
                                    let vtable_cap = gsm!().alloc_object::<VTableObj>(12).unwrap();
                                    kprintln!("miss table level {} addr {:x}", level, vaddr);
                                    child.vspace.map_table(vtable_cap, vaddr, level).unwrap();
                                }
                            }
                        };

                        let frame_addr = gsm!().insert_ram_at(frame_parent_cap.clone(), 0, Permission::writable());
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

        let child_tcb = gsm!().alloc_object::<TcbObj>(12).unwrap();

        let start_addr = e.header().entry_point() as usize;

        child_tcb.configure(child_root_vn.slot, child_root_cn.slot)
           .expect("Error Configuring TCB");
        child_tcb.set_registers(0b1100, start_addr, 0x8000000)
           .expect("Error Setting Registers");
        kprintln!("before spawning child process");
        child_tcb.resume()
           .expect("Error Resuming TCB");
        kprintln!("after spawning child process");
    } else {
        unimplemented!("Elf32 binary is not supported!");
    }

    Ok(())
}