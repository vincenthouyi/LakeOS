use crate::objects::{CNodeObj, EpCap, RamObj, TcbCap, TcbObj, UntypedObj, VTableObj};
use rustyl4api::vspace::Permission;
use crate::spaceman::vspace_man::{VSpaceMan, VSpaceEntry};

#[derive(Debug)]
pub struct ProcessBuilder<'a> {
    elf: &'a [u8],
    stdin: Option<EpCap>,
    stdout: Option<EpCap>,
    stderr: Option<EpCap>,
    name_server: Option<EpCap>,
}

#[allow(dead_code)]
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
            name_server: None,
        }
    }

    pub fn stdin(mut self, ep: EpCap) -> Self {
        self.stdin = Some(ep);
        self
    }

    pub fn stdout(mut self, ep: EpCap) -> Self {
        self.stdout = Some(ep);
        self
    }

    pub fn stderr(mut self, ep: EpCap) -> Self {
        self.stderr = Some(ep);
        self
    }

    pub fn name_server(mut self, ep: EpCap) -> Self {
        self.name_server = Some(ep);
        self
    }

    pub fn spawn(self) -> Result<Child, ()> {
        use crate::space_manager::gsm;
        use crate::objects::cnode::CNODE_ENTRY_SZ;
        use crate::objects::tcb::TCB_OBJ_BIT_SZ;
        use rustyl4api::process::{ProcessCSpace, PROCESS_ROOT_CNODE_SIZE};
        use rustyl4api::vspace::{FRAME_BIT_SIZE, FRAME_SIZE};

        let rootcn_bitsz = (PROCESS_ROOT_CNODE_SIZE * CNODE_ENTRY_SZ).trailing_zeros() as usize;
        let child_tcb = gsm!().alloc_object::<TcbObj>(TCB_OBJ_BIT_SZ).unwrap();
        let child_root_cn = gsm!().alloc_object::<CNodeObj>(rootcn_bitsz).unwrap();
        let child_root_vn = gsm!().alloc_object::<VTableObj>(12).unwrap();
        let child_root_vn_slot = child_root_vn.slot;
        let vspace = VSpaceMan::new(child_root_vn);

        let mut cur_free = ProcessCSpace::ProcessFixedMax as usize;

        let entry = elfloader::load_elf(self.elf, 0x8000000, 4096, &mut |vrange, flags| {
            use crate::objects::RamCap;

            let vaddr = vrange.start as usize;
            let perm = Permission::new(flags & 0b100 != 0, flags & 0b010 != 0, flags & 0b001 != 0);
            let frame_cap = gsm!().alloc_object::<RamObj>(FRAME_BIT_SIZE).unwrap();
            let frame_cap_slot = frame_cap.slot;
            let frame_parent_slot = gsm!().cspace_alloc().unwrap();
            frame_cap.derive(frame_parent_slot).unwrap();
            let frame_parent_cap = RamCap::new(frame_parent_slot);

            let mut frame_entry = VSpaceEntry::new_frame(frame_cap, vaddr, perm, 4);
            while let Err((e, ent)) = vspace.install_entry(frame_entry, true) {
                frame_entry = ent;
                match e {
                    // VSpaceManError::SlotOccupied{level} => {
                    //     panic!("slot occupied at level {} vaddr {:x}", level, vaddr);
                    // }
                    // VSpaceManError::SlotTypeError{level} => {
                    //     panic!("wrong slot type at level {} vaddr {:x}", level, vaddr);
                    // }
                    // VSpaceManError::PageTableMiss{level} => {
                    crate::spaceman::vspace_man::VSpaceManError::PageTableMiss { level } => {
                        let vtable_cap = gsm!().alloc_object::<VTableObj>(12).unwrap();
                        let vtable_entry = VSpaceEntry::new_table(vtable_cap, vaddr, level + 1);
                        let vtable_cap_slot = vtable_entry.cap_slot();
                        vspace.install_entry(vtable_entry, true)
                            .unwrap();
                        child_root_cn
                            .cap_copy(cur_free, vtable_cap_slot)
                            .unwrap();
                        cur_free += 1;
                    }
                    e => {
                        panic!("vaddr {:x} perm {:?} error: {:?}", vaddr, perm, e);
                    }
                }
            }

            let frame_addr =
                gsm!().insert_ram_at(frame_parent_cap, 0, Permission::writable());
            let frame = unsafe { core::slice::from_raw_parts_mut(frame_addr, FRAME_SIZE) };
            child_root_cn
                .cap_copy(cur_free, frame_cap_slot)
                .unwrap();
            cur_free += 1;
            frame
        })
        .map_err(|_| ())?;

        child_tcb
            .configure(Some(child_root_vn_slot), Some(child_root_cn.slot))
            .expect("Error Configuring TCB");
        child_tcb
            .set_registers(0b1100, entry as usize, 0x8000000)
            .expect("Error Setting Registers");
        child_root_cn
            .cap_copy(ProcessCSpace::TcbCap as usize, child_tcb.slot)
            .map_err(|_| ())?;
        child_root_cn
            .cap_copy(ProcessCSpace::RootCNodeCap as usize, child_root_cn.slot)
            .map_err(|_| ())?;
        child_root_cn
            .cap_copy(ProcessCSpace::RootVNodeCap as usize, child_root_vn_slot)
            .map_err(|_| ())?;
        child_root_cn
            .cap_copy(
                ProcessCSpace::Stdin as usize,
                self.stdin.as_ref().unwrap().slot,
            )
            .map_err(|_| ())?;
        child_root_cn
            .cap_copy(
                ProcessCSpace::Stdout as usize,
                self.stdout.as_ref().unwrap().slot,
            )
            .map_err(|_| ())?;
        child_root_cn
            .cap_copy(
                ProcessCSpace::Stderr as usize,
                self.stderr.as_ref().unwrap().slot,
            )
            .map_err(|_| ())?;
        child_root_cn
            .cap_copy(
                ProcessCSpace::NameServer as usize,
                self.name_server.as_ref().unwrap().slot,
            )
            .map_err(|_| ())?;
        let init_untyped = gsm!().alloc_object::<UntypedObj>(18).ok_or(())?;
        child_root_cn
            .cap_copy(ProcessCSpace::InitUntyped as usize, init_untyped.slot)
            .map_err(|_| ())?;

        child_tcb.resume().expect("Error Resuming TCB");

        Ok(Child {
            vspace: vspace,
            tcb: child_tcb,
            stdin: self.stdin.unwrap(),
            stdout: self.stdout.unwrap(),
            stderr: self.stderr.unwrap(),
        })
    }
}
