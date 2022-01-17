#[naked]
#[no_mangle]
#[link_section = ".text.startup"]
#[allow(named_asm_labels)]
unsafe extern "C" fn _start() {
    const TCR_T0SZ: usize = (64 - 48) << 0;
    const TCR_T1SZ: usize = (64 - 48) << 16;
    const TCR_TG0_4K: usize = 0 << 14;
    const TCR_TG1_4K: usize = 2 << 30;
    const TCR_A1: usize = 0 << 22; // Use TTBR0_EL1.ASID as ASID
    const TCR_AS: usize = 1 << 36; // 16 bit ASID size
    const TCR_IRGN_WBWA: usize = (1 << 8) | (1 << 24);
    const TCR_ORGN_WBWA: usize = (1 << 10) | (1 << 26);
    const TCR_SHARED: usize = (3 << 12) | (3 << 28);
    const TCR_VALUE: usize = TCR_T0SZ
        | TCR_T1SZ
        | TCR_TG0_4K
        | TCR_TG1_4K
        | TCR_AS
        | TCR_A1
        | TCR_IRGN_WBWA
        | TCR_ORGN_WBWA
        | TCR_SHARED;

    const CONTROL_I: usize = 12; // Instruction access Cacheability control
    const CONTROL_C: usize = 2; // Cacheability control, for data accesses
    const CONTROL_M: usize = 0; // MMU enable
                                // const  CONTROL_A : usize = 1;  // Alignment check
    const SCTLR_VALUE: usize = BIT!(CONTROL_I) | BIT!(CONTROL_C) | BIT!(CONTROL_M); // TODO: enable alignment check
    asm!(r#"
    msr     daifset, 0xf
    mrs     x0, mpidr_el1        // check core id, only one core is used.
    mov     x1, #0xc1000000
    bic     x0, x0, x1
    cbz     x0, zero_bss
hang:
    b       hang

zero_bss:
    // load the start address and number of bytes in BSS section
    ldr     x1, =__bss_start
    ldr     x2, =__bss_length

zero_bss_loop:
    // zero out the BSS section, 64-bits at a time
    cbz     x2, jump_to_el1 
    str     xzr, [x1], #8
    sub     x2, x2, #8
    cbnz    x2, zero_bss_loop

jump_to_el1:
    // /* stack pointer = kernel_stack + ((cpu_id + 1) * 4096) */
    // /* x0 stored core id already */
    // ldr     x1, =4096
    // mul     x1, x1, x0
    // ldr     x2, =kernel_stack + 4096
    // add     x2, x2, x1
    // msr     sp_el1, x2
    ldr     x1, =0x80000
    mov     sp, x1

    /* Store (kernel_stack | cpu_id) in tpidr_el1 */
    orr     x0, x0, x2
    msr     tpidr_el1, x0

    // mov     x0, #3 << 20
    // msr     cpacr_el1, x0        // enable fp/simd at el1

    // initialize hcr_el2
    mov     x0, #(1 << 31)
    msr     hcr_el2, x0          // set el1 to 64 bit

    /* put spsr to a known state */
    mov     x0, #(15 << 6 | 0b01 << 2 | 1) // DAIF masked, EL1, SpSelx 
    msr     spsr_el2, x0

    // /* set up exception handlers (guide: 10.4) */
    // ldr     x2, =trap_vectors
    // msr     VBAR_EL1, x2

    /* Translation Control Register */
    ldr     x4, ={TCR_VALUE}
    msr     tcr_el1, x4
    isb

    // /* Initialize page tables */
    // // HACK: set a temp sp in case init_cpu use stack
    // mov     sp, #0x80000
    // bl      init_cpu

    /* Initialize SCTLR_EL1 */
    ldr     x0, ={SCTLR_VALUE}
    msr     sctlr_el1, x0
    isb

    // ldr     x0, =bootloader_main
    // msr     elr_el2, x0
    b bootloader_main

    /* jump to kmain in higher half runing in el1 */
    eret
    "#,
    TCR_VALUE = const TCR_VALUE,
    SCTLR_VALUE = const SCTLR_VALUE,
    options(noreturn))
}

use crate::boot_info::{BootInfo, BootInfoEntry, RamInfo, RamType};
use crate::ram_block;
use elf_loader::ElfLoader;
use elf_rs::{Elf, ElfFile, ProgramHeaderWrapper};
use log::info;
use vspace::{
    arch::Aarch64PageTableEntry,
    arch::{Level1, Level2, Level3, Level4},
    PhysAddr, VirtAddr,
};

type VSpace<'a> = vspace::arch::VSpace<'a, 0>;

const KERNEL_OFFSET: usize = 0xffff0000_00000000;
static INIT_FS: &[u8] = include_bytes!("../build/initfs.cpio");

struct KernelLoader<'a> {
    ram_blocks: &'a mut ram_block::RamBlockList<4>,
    boot_info: &'a mut BootInfo,
    vspace: &'a mut VSpace<'a>,
}

impl<'a> KernelLoader<'a> {
    pub fn frame_alloc(&mut self, size: usize, frame_type: RamType) -> Option<*mut u8> {
        let frame = self.ram_blocks.frame_alloc(size)?;
        let bi_entry = BootInfoEntry::RamEntry(RamInfo {
            base: frame as usize,
            size: size,
            mem_type: frame_type,
        });
        self.boot_info.append_entry(bi_entry);
        return Some(frame);
    }
}

pub const fn align_down(addr: usize, align: usize) -> usize {
    addr & !(align - 1)
}

pub const fn align_up(addr: usize, align: usize) -> usize {
    align_down(addr.saturating_add(align - 1), align)
}

impl<'a> ElfLoader for KernelLoader<'a> {
    fn allocate(
        &mut self,
        load_headers: &mut dyn Iterator<Item = ProgramHeaderWrapper>,
    ) -> Result<(), &'static str> {
        for header in load_headers {
            let base = align_down(header.vaddr() as usize, 4096);
            let top = align_up((header.vaddr() + header.memsz()) as usize, 4096);

            for page_base in (base..top).step_by(512 * 1024 * 1024 * 1024) {
                let vaddr = VirtAddr(page_base);
                if self
                    .vspace
                    .lookup_slot::<Level4>(vaddr)
                    .map(|slot| !slot.is_valid())
                    .unwrap()
                {
                    let pud_paddr = self
                        .frame_alloc(4096, RamType::KernelPageTable)
                        .expect("allocating frame failed");
                    let pgde_entry =
                        Aarch64PageTableEntry::table_entry(PhysAddr(pud_paddr as usize));
                    self.vspace.map_entry::<Level4>(vaddr, pgde_entry).unwrap();
                }
            }

            for page_base in (base..top).step_by(1 * 1024 * 1024 * 1024) {
                let vaddr = VirtAddr(page_base);
                if self
                    .vspace
                    .lookup_slot::<Level3>(vaddr)
                    .map(|slot| !slot.is_valid())
                    .unwrap()
                {
                    let pud_paddr = self
                        .frame_alloc(4096, RamType::KernelPageTable)
                        .expect("allocating frame failed");
                    let pgde_entry =
                        Aarch64PageTableEntry::table_entry(PhysAddr(pud_paddr as usize));
                    self.vspace.map_entry::<Level3>(vaddr, pgde_entry).unwrap();
                }
            }

            for page_base in (base..top).step_by(2 * 1024 * 1024) {
                let vaddr = VirtAddr(page_base);
                if self
                    .vspace
                    .lookup_slot::<Level2>(vaddr)
                    .map(|slot| !slot.is_valid())
                    .unwrap()
                {
                    let pd_paddr = self
                        .frame_alloc(4096, RamType::KernelPageTable)
                        .expect("allocating frame failed");
                    let pude_entry =
                        Aarch64PageTableEntry::table_entry(PhysAddr(pd_paddr as usize));
                    self.vspace.map_entry::<Level2>(vaddr, pude_entry).unwrap();
                }
            }

            for page_base in (base..top).step_by(4 * 1024) {
                let frame_ptr = self
                    .frame_alloc(4 * 1024, RamType::KernelPage)
                    .expect("allocating frame failed");
                let frame_addr = PhysAddr(frame_ptr as usize);
                let page_entry = Aarch64PageTableEntry::page_entry::<Level1>(
                    frame_addr,
                    true,
                    true,
                    true,
                    vspace::arch::mmu::Shareability::InnerSharable,
                    vspace::arch::mmu::AccessPermission::KernelOnly,
                    vspace::arch::mmu::MemoryAttr::Normal,
                );
                let vaddr = VirtAddr(page_base);
                self.vspace
                    .map_entry::<Level1>(vaddr, page_entry)
                    .expect("Installing kernel frame failed");
            }
        }
        Ok(())
    }

    // fn relocate(&mut self, _entry: &Rela<P64>) -> Result<(), &'static str> {
    //     unimplemented!()
    // }

    fn load(&mut self, program_header: ProgramHeaderWrapper) -> Result<(), &'static str> {
        let base = program_header.vaddr();
        let mut vaddr = align_down(base as usize, 4096);
        let mut region_offset = 0;
        let mut frame_offset = (base as usize) % 4096;
        let region = program_header.content();

        while region_offset < region.len() {
            let frame_base = self.vspace.paddr_of_vaddr(VirtAddr(vaddr)).unwrap();
            let frame = unsafe { core::slice::from_raw_parts_mut(frame_base.0 as *mut u8, 4096) };
            let copy_len = (region.len() - region_offset).min(4096) - frame_offset;
            frame[frame_offset..frame_offset + copy_len]
                .copy_from_slice(&region[region_offset..region_offset + copy_len]);

            region_offset += copy_len;
            frame_offset = (frame_offset + copy_len) % 4096;
            vaddr += 4096;
        }
        Ok(())
    }
}

fn map_kernel_virtual_address_space<const O: usize>(
    vspace: &mut VSpace,
    ram_blocks: &mut ram_block::RamBlockList<4>,
) {
    pub const PHYS_IO_BASE: usize = 0x3f000000;
    pub const IO_BASE: usize = PHYS_IO_BASE + KERNEL_OFFSET;

    let pud_paddr = ram_blocks.frame_alloc(4096).expect("alloc pud failed");
    let pud_entry = Aarch64PageTableEntry::table_entry(PhysAddr(pud_paddr as usize));
    vspace
        .map_entry::<Level4>(VirtAddr(KERNEL_OFFSET), pud_entry)
        .expect("mapping pud failed");

    let pd_paddr = ram_blocks.frame_alloc(4096).expect("alloc pd failed");
    let pd_entry = Aarch64PageTableEntry::table_entry(PhysAddr(pd_paddr as usize));
    vspace
        .map_entry::<Level3>(VirtAddr(KERNEL_OFFSET), pd_entry)
        .expect("mapping pd failed");

    for vaddr in (KERNEL_OFFSET..IO_BASE).step_by(2 * 1024 * 1024) {
        let frame_entry = Aarch64PageTableEntry::page_entry::<Level2>(
            PhysAddr(vaddr - KERNEL_OFFSET),
            true,
            true,
            true,
            vspace::arch::mmu::Shareability::InnerSharable,
            vspace::arch::mmu::AccessPermission::KernelOnly,
            vspace::arch::mmu::MemoryAttr::Normal,
        );
        vspace
            .map_entry::<Level2>(VirtAddr(vaddr), frame_entry)
            .expect("mapping normal pde failed");
    }

    for vaddr in (IO_BASE..0x40000000 + KERNEL_OFFSET).step_by(2 * 1024 * 1024) {
        let frame_entry = Aarch64PageTableEntry::page_entry::<Level2>(
            PhysAddr(vaddr - KERNEL_OFFSET),
            true,
            true,
            true,
            vspace::arch::mmu::Shareability::InnerSharable,
            vspace::arch::mmu::AccessPermission::KernelOnly,
            vspace::arch::mmu::MemoryAttr::DevicenGnRnE,
        );
        vspace
            .map_entry::<Level2>(VirtAddr(vaddr), frame_entry)
            .expect("mapping device pde failed");
    }

    {
        let frame_entry = Aarch64PageTableEntry::page_entry::<Level3>(
            PhysAddr(0x40000000),
            true,
            true,
            true,
            vspace::arch::mmu::Shareability::InnerSharable,
            vspace::arch::mmu::AccessPermission::KernelOnly,
            vspace::arch::mmu::MemoryAttr::DevicenGnRnE,
        );
        vspace
            .map_entry::<vspace::arch::Level3>(VirtAddr(0x40000000 + KERNEL_OFFSET), frame_entry)
            .expect("mapping device pude failed");
    }
}

unsafe fn jump_to_kernel(kernel_start: usize, bi_frame: *mut u8) -> ! {
    // ABI
    // x0: BootInfo frame base address

    asm!("
    msr     elr_el2, {0}

    /* jump to kernel entry in higher half runing in el1 */
    eret
    ",
    in(reg) kernel_start,
    in("x0") bi_frame,
    options(noreturn))
}

#[no_mangle]
pub extern "C" fn bootloader_main() -> ! {
    crate::uart::init_uart();

    let mut ram_blocks = ram_block::RamBlockList::<4>::new();

    for atag in atags::Atags::get(0) {
        if let atags::Atag::Mem(mem) = atag {
            info!(
                "Reading Atag Mem {:x?}, range 0x{:x}-0x{:x}",
                mem,
                mem.start,
                mem.start + mem.size
            );
            ram_blocks.insert(mem.start as usize, mem.size as usize)
        }
    }

    //TODO: avoid using bootloader mem
    let bootloader_upper_bound = unsafe { align_up(&crate::_end as *const _ as usize, 4096) };
    ram_blocks.frame_alloc(bootloader_upper_bound);

    let init_fs = cpio::NewcReader::from_bytes(INIT_FS);
    let kernel_elf = init_fs
        .entries()
        .find(|e| e.name() == b"rustyl4")
        .map(|e| Elf::from_bytes(e.content()).unwrap())
        .expect("kernel not found in init fs!");

    let bi_frame = ram_blocks
        .frame_alloc(4096)
        .expect("Allocating BootInfo frame failed!");
    let mut boot_info = unsafe { BootInfo::new_from_frame(bi_frame, true) };
    boot_info.append_entry(BootInfoEntry::RamEntry(RamInfo {
        base: _start as usize,
        size: align_up(
            unsafe { &crate::_end as *const _ as usize } - _start as usize,
            4096,
        ),
        mem_type: RamType::BootLoader,
    }));
    boot_info.append_entry(BootInfoEntry::RamEntry(RamInfo {
        base: &INIT_FS[0] as *const _ as usize,
        size: align_up(INIT_FS.len(), 4096),
        mem_type: RamType::InitRamFS,
    }));

    let vspace_root_paddr = ram_blocks
        .frame_alloc(4096)
        .expect("Allocating root vspace frame failed!");
    // let vspace_root =
    //     unsafe { vspace::Table::<vspace::arch::TopLevel>::from_vaddr(vspace_root_paddr) };
    let mut vspace = unsafe { VSpace::from_vaddr(vspace_root_paddr) };
    map_kernel_virtual_address_space::<0>(&mut vspace, &mut ram_blocks);

    let mut kernel_loader = KernelLoader {
        ram_blocks: &mut ram_blocks,
        boot_info: &mut boot_info,
        vspace: &mut vspace,
    };

    kernel_loader
        .load_elf(&kernel_elf)
        .expect("fail to load kernel!");
    info!("load finished");

    for blk in ram_blocks.list.iter() {
        match blk {
            Some(b) => {
                if b.remain() > 0 {
                    boot_info.append_entry(BootInfoEntry::RamEntry(RamInfo {
                        base: b.cur,
                        size: b.top() - b.cur,
                        mem_type: RamType::FreeSpace,
                    }))
                }
            }
            _ => {}
        }
    }

    unsafe {
        vspace::arch::mmu::install_kernel_vspace(PhysAddr(vspace_root_paddr as usize));
    }

    unsafe { jump_to_kernel(kernel_elf.entry_point() as usize, bi_frame) }
}
