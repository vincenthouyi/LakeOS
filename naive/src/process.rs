use rustyl4api::object::{VTableObj, RamObj, CNodeObj, TcbObj, EpCap, TcbCap, UntypedObj};
use rustyl4api::vspace::Permission;
use spaceman::vspace_man::{VSpaceMan, VSpaceManError};

#[derive(Debug)]
pub struct ProcessBuilder<'a> {
    elf: &'a [u8],
    stdin: Option<EpCap>,
    stdout: Option<EpCap>,
    stderr: Option<EpCap>,
}

pub struct Child {
    vspace: VSpaceMan,
    tcb: TcbCap,
    stdin: EpCap,
    stdout: EpCap,
    stderr: EpCap,
}

impl<'a> ProcessBuilder<'a> {
    pub fn new(elf: &'a [u8]) -> Self {
        Self {
            elf: elf,
            stdin: None,
            stdout: None,
            stderr: None,
        }
    }

    pub fn stdin(&mut self, ep: EpCap) -> &mut Self {
        self.stdin = Some(ep);
        self
    }

    pub fn stdout(&mut self, ep: EpCap) -> &mut Self {
        self.stdout = Some(ep);
        self
    }

    pub fn stderr(&mut self, ep: EpCap) -> &mut Self {
        self.stderr = Some(ep);
        self
    }

    pub fn spawn(&mut self) -> Result<Child, ()> {
        use rustyl4api::object::cnode::{CNODE_ENTRY_SZ};
        use rustyl4api::object::tcb::TCB_OBJ_BIT_SZ;
        use elf_rs::{Elf, ProgramType};
        use rustyl4api::vspace::{FRAME_BIT_SIZE, FRAME_SIZE};
        use crate::space_manager::gsm;
        use rustyl4api::process::{ProcessCSpace, PROCESS_ROOT_CNODE_SIZE};

        let elf = Elf::from_bytes(self.elf).map_err(|_| ())?;

        let rootcn_bitsz = (PROCESS_ROOT_CNODE_SIZE * CNODE_ENTRY_SZ).trailing_zeros() as usize;
        let child_tcb = gsm!().alloc_object::<TcbObj>(TCB_OBJ_BIT_SZ).unwrap();
        let child_root_cn = gsm!().alloc_object::<CNodeObj>(rootcn_bitsz).unwrap();
        let child_root_vn = gsm!().alloc_object::<VTableObj>(12).unwrap();
        let mut vspace = VSpaceMan::new(child_root_vn.clone());

        let mut cur_free = ProcessCSpace::ProcessFixedMax as usize;
        if let Elf::Elf64(e) = elf {
            for ph in e.program_header_iter()
            {
                // kprintln!("{:?}", ph);
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
                        let section = &self.elf[sec_base.. sec_base + sec_len];

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

                                while let Err(e) = vspace.map_frame(frame_cap.clone(), vaddr, perm, 4) {
                                    match e {
                                        // VSpaceManError::SlotOccupied{level} => {
                                        //     panic!("slot occupied at level {} vaddr {:x}", level, vaddr);
                                        // }
                                        // VSpaceManError::SlotTypeError{level} => {
                                        //     panic!("wrong slot type at level {} vaddr {:x}", level, vaddr);
                                        // }
                                        // VSpaceManError::PageTableMiss{level} => {
                                        rustyl4api::error::SysError::VSpaceTableMiss{level} => {
                                            let vtable_cap = gsm!().alloc_object::<VTableObj>(12).unwrap();
                                            // kprintln!("miss table level {} addr {:x}", level, vaddr);
                                            vspace.map_table(vtable_cap.clone(), vaddr, level as usize).unwrap();
                                            child_root_cn.cap_copy(cur_free, vtable_cap.slot).map_err(|_| ()).unwrap();
                                            cur_free += 1;
                                        }
                                        e => {
                                            panic!()
                                        }
                                    }
                                };

                                child_root_cn.cap_copy(cur_free, frame_cap.slot).map_err(|_| ()).unwrap();
                                cur_free += 1;
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

                            while let Err(e) = vspace.map_frame(frame_cap.clone(), vaddr, perm, 4) {
                                match e {
                                    // VSpaceManError::SlotOccupied{level} => {
                                    //     panic!("slot occupied at level {} vaddr {:x}", level, vaddr);
                                    // }
                                    // VSpaceManError::SlotTypeError{level} => {
                                    //     panic!("wrong slot type at level {} vaddr {:x}", level, vaddr);
                                    // }
                                    // VSpaceManError::PageTableMiss{level} => {
                                    rustyl4api::error::SysError::VSpaceTableMiss{level} => {
                                        let vtable_cap = gsm!().alloc_object::<VTableObj>(12).unwrap();
                                        // kprintln!("miss table level {} addr {:x}", level, vaddr);
                                        vspace.map_table(vtable_cap.clone(), vaddr, level as usize).unwrap();
                                        child_root_cn.cap_copy(cur_free, vtable_cap.slot).map_err(|_| ()).unwrap();
                                        cur_free += 1;
                                    }
                                    _ => {
                                        panic!()
                                    }
                                }
                            };

                            child_root_cn.cap_copy(cur_free, frame_cap.slot).map_err(|_| ()).unwrap();
                            cur_free += 1;
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

            let start_addr = e.header().entry_point() as usize;

            child_tcb.configure(Some(child_root_vn.slot), Some(child_root_cn.slot))
                .expect("Error Configuring TCB");
            child_tcb.set_registers(0b1100, start_addr, 0x8000000)
                .expect("Error Setting Registers");
        } else {
            unimplemented!("Elf32 binary is not supported!");
        }

        child_root_cn.cap_copy(ProcessCSpace::TcbCap as usize, child_tcb.slot).map_err(|_| ())?;
        child_root_cn.cap_copy(ProcessCSpace::RootCNodeCap as usize, child_root_cn.slot).map_err(|_| ())?;
        child_root_cn.cap_copy(ProcessCSpace::RootVNodeCap as usize, child_root_vn.slot).map_err(|_| ())?;
        child_root_cn.cap_copy(ProcessCSpace::Stdin as usize, self.stdin.as_ref().unwrap().slot).map_err(|_| ())?;
        child_root_cn.cap_copy(ProcessCSpace::Stdout as usize, self.stdout.as_ref().unwrap().slot).map_err(|_| ())?;
        child_root_cn.cap_copy(ProcessCSpace::Stderr as usize, self.stderr.as_ref().unwrap().slot).map_err(|_| ())?;
        let init_untyped = gsm!().alloc_object::<UntypedObj>(16).ok_or(())?;
        child_root_cn.cap_copy(ProcessCSpace::InitUntyped as usize, init_untyped.slot).map_err(|_| ())?;

        child_tcb.resume()
            .expect("Error Resuming TCB");

        Ok(Child {
            vspace: vspace,
            tcb: child_tcb,
            stdin: self.stdin.as_ref().unwrap().clone(),
            stdout: self.stdout.as_ref().unwrap().clone(),
            stderr: self.stderr.as_ref().unwrap().clone(),
        })
    }
}