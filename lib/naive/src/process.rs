use elfloader::{ElfLoader, ElfBinary, LoadableHeaders, Rela, P64, Flags, VAddr};

use rustyl4api::vspace::Permission;
use rustyl4api::process::{ProcessCSpace, PROCESS_ROOT_CNODE_SIZE, PROCESS_MAIN_THREAD_STACK_TOP, PROCESS_MAIN_THREAD_STACK_PAGES};
use rustyl4api::vspace::{FRAME_BIT_SIZE, FRAME_SIZE};

use crate::objects::{CNodeObj, EpCap, RamObj, TcbCap, TcbObj, UntypedObj, VTableObj, CNodeRef, VTableRef, UntypedCap};
use crate::spaceman::vspace_man::{VSpaceMan, VSpaceEntry, VSpaceManError};
use crate::space_manager::copy_cap;
use crate::space_manager::gsm;
use crate::objects::cnode::CNODE_ENTRY_SZ;
use crate::objects::tcb::TCB_OBJ_BIT_SZ;
use crate::utils::align_down;

struct ProcessElfLoader<'a> {
    vspace: &'a VSpaceMan,
    child_root_cn: &'a CNodeRef,
    cur_free: &'a mut usize,
}

impl<'a> ElfLoader for ProcessElfLoader<'a> {
    fn allocate(&mut self, load_headers: LoadableHeaders) -> Result<(), &'static str> {
        for header in load_headers {
            let flags = header.flags();
            let perm = Permission::new(flags.is_read(), flags.is_write(), flags.is_execute());
            let base = align_down(header.virtual_addr() as usize, FRAME_SIZE);
            let top = (header.virtual_addr() + header.mem_size()) as usize;
            for page_base in (base .. top).step_by(FRAME_SIZE) {
                let frame_cap = gsm!().alloc_object::<RamObj>(FRAME_BIT_SIZE).unwrap().into();
                let mut frame_entry = VSpaceEntry::new_frame(frame_cap, page_base, perm, 4);
                while let Err((e, ent)) = self.vspace.install_entry(frame_entry, true) {
                    frame_entry = ent;
                    match e {
                        VSpaceManError::PageTableMiss { level } => {
                            let vtable_cap: VTableRef = gsm!().alloc_object::<VTableObj>(12).unwrap().into();
                            let vtable_entry = VSpaceEntry::new_table(vtable_cap.clone(), page_base, level + 1);
                            self.vspace.install_entry(vtable_entry, true)
                                .unwrap();
                            self.child_root_cn
                                .cap_copy(*self.cur_free, vtable_cap.slot.slot())
                                .unwrap();
                            *self.cur_free += 1;
                        }
                        e => {
                            panic!("vaddr {:x} perm {:?} error: {:?}", page_base, perm, e);
                        }
                    }
                }
            }
        }
        Ok(())
    }

    fn relocate(&mut self, _entry: &Rela<P64>) -> Result<(), &'static str> {
        unimplemented!()
    }

    fn load(&mut self, _flags: Flags, base: VAddr, region: &[u8]) -> Result<(), &'static str> {
        let mut vaddr = align_down(base as usize, FRAME_SIZE);
        let mut region_offset = 0;
        let mut frame_offset = (base as usize) % FRAME_SIZE;

        while region_offset < region.len() {
            let frame = self.vspace.lookup_entry(vaddr, 4).unwrap();
            let frame_parent_cap = copy_cap(&frame.as_frame_node().unwrap().cap).unwrap();
            let frame_addr = gsm!().insert_ram_at(frame_parent_cap, 0, Permission::writable());
            let frame = unsafe {
                core::slice::from_raw_parts_mut(frame_addr, FRAME_SIZE)
            };
            let copy_len = (region.len() - region_offset).min(FRAME_SIZE) - frame_offset;
            frame[frame_offset .. frame_offset + copy_len].copy_from_slice(&region[region_offset .. region_offset + copy_len]);
            gsm!().memory_unmap(frame_addr, FRAME_SIZE);

            region_offset += copy_len;
            frame_offset = (frame_offset + copy_len) % FRAME_SIZE;
            vaddr += FRAME_SIZE;
        }
        Ok(())
    }
}

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
    rootcn: CNodeRef,
    init_untyped: UntypedCap,
    name_server: EpCap,
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

        let rootcn_bitsz = (PROCESS_ROOT_CNODE_SIZE * CNODE_ENTRY_SZ).trailing_zeros() as usize;
        let child_tcb = gsm!().alloc_object::<TcbObj>(TCB_OBJ_BIT_SZ).unwrap();
        let child_root_cn: CNodeRef = gsm!().alloc_object::<CNodeObj>(rootcn_bitsz).unwrap().into();
        let child_root_vn: VTableRef = gsm!().alloc_object::<VTableObj>(12).unwrap().into();
        let vspace = VSpaceMan::new(child_root_vn.clone());

        let mut cur_free = ProcessCSpace::WellKnownMax as usize;

        let mut process_elf_loader = ProcessElfLoader {
            vspace: &vspace,
            child_root_cn: &child_root_cn,
            cur_free: &mut cur_free,
        };

        let child_elf = ElfBinary::new("process", self.elf).unwrap();

        child_elf.load(&mut process_elf_loader).unwrap();
        for i in 1 .. PROCESS_MAIN_THREAD_STACK_PAGES + 1 {
            let page_base = PROCESS_MAIN_THREAD_STACK_TOP - i * FRAME_SIZE;
            let perm = Permission::writable();
            let frame_cap = gsm!().alloc_object::<RamObj>(FRAME_BIT_SIZE).unwrap().into();
            let mut frame_entry = VSpaceEntry::new_frame(frame_cap, page_base, perm, 4);
            while let Err((e, ent)) = vspace.install_entry(frame_entry, true) {
                frame_entry = ent;
                match e {
                    VSpaceManError::PageTableMiss { level } => {
                        let vtable_cap: VTableRef = gsm!().alloc_object::<VTableObj>(12).unwrap().into();
                        let vtable_entry = VSpaceEntry::new_table(vtable_cap.clone(), page_base, level + 1);
                        vspace.install_entry(vtable_entry, true)
                            .unwrap();
                        child_root_cn
                            .cap_copy(cur_free, vtable_cap.slot.slot())
                            .unwrap();
                        cur_free += 1;
                    }
                    e => {
                        panic!("vaddr {:x} perm {:?} error: {:?}", page_base, perm, e);
                    }
                }
            }
        }
        let entry = child_elf.entry_point() as usize;

        child_tcb
            .configure(Some(&child_root_vn), Some(&child_root_cn))
            .expect("Error Configuring TCB");
        child_tcb
            .set_registers(0b1100, entry as usize, PROCESS_MAIN_THREAD_STACK_TOP)
            .expect("Error Setting Registers");
        child_root_cn
            .cap_copy(ProcessCSpace::TcbCap as usize, child_tcb.slot.slot())
            .map_err(|_| ())?;
        child_root_cn
            .cap_copy(ProcessCSpace::RootCNodeCap as usize, child_root_cn.slot.slot())
            .map_err(|_| ())?;
        child_root_cn
            .cap_copy(ProcessCSpace::RootVNodeCap as usize, child_root_vn.slot.slot())
            .map_err(|_| ())?;
        child_root_cn
            .cap_copy(
                ProcessCSpace::Stdin as usize,
                self.stdin.as_ref().unwrap().slot.slot(),
            )
            .map_err(|_| ())?;
        child_root_cn
            .cap_copy(
                ProcessCSpace::Stdout as usize,
                self.stdout.as_ref().unwrap().slot.slot(),
            )
            .map_err(|_| ())?;
        child_root_cn
            .cap_copy(
                ProcessCSpace::Stderr as usize,
                self.stderr.as_ref().unwrap().slot.slot(),
            )
            .map_err(|_| ())?;
        child_root_cn
            .cap_copy(
                ProcessCSpace::NameServer as usize,
                self.name_server.as_ref().unwrap().slot.slot(),
            )
            .map_err(|_| ())?;
        let init_untyped = gsm!().alloc_object::<UntypedObj>(18).ok_or(())?;
        child_root_cn
            .cap_copy(ProcessCSpace::InitUntyped as usize, init_untyped.slot.slot())
            .map_err(|_| ())?;

        child_tcb.resume().expect("Error Resuming TCB");

        Ok(Child {
            vspace: vspace,
            tcb: child_tcb,
            stdin: self.stdin.unwrap(),
            stdout: self.stdout.unwrap(),
            stderr: self.stderr.unwrap(),
            rootcn: child_root_cn,
            init_untyped,
            name_server: self.name_server.unwrap(),
        })
    }
}
