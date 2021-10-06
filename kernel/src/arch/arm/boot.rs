use crate::prelude::*;
use core::cell::{Cell, UnsafeCell};
use core::convert::TryFrom;
use core::mem::MaybeUninit;

use crate::arch::cpuid;
use crate::objects::*;
use crate::utils::percore::PerCore;
use crate::NCPU;
use sysapi::init::InitCSpaceSlot::*;
use sysapi::process::{
    ProcessCSpace, PROCESS_MAIN_THREAD_STACK_PAGES, PROCESS_MAIN_THREAD_STACK_TOP,
    PROCESS_ROOT_CNODE_SIZE,
};
use sysapi::vspace::{Permission, FRAME_BIT_SIZE, FRAME_SIZE};

use bootloader::boot_info::{BootInfo, BootInfoEntry, RamType};
use elfloader::{ElfBinary, ElfLoader, Flags, LoadableHeaders, Rela, VAddr, P64};
use vspace::{Level1, Level2, Level3, Level4, TableLevel, VirtAddr};

#[derive(Copy, Clone)]
#[repr(align(4096))]
struct Frame([usize; FRAME_SIZE]);

#[no_mangle]
static mut kernel_stack: [Frame; NCPU] = [Frame([0; FRAME_SIZE]); NCPU];

static mut INIT_CNODE: MaybeUninit<[CNodeEntry; PROCESS_ROOT_CNODE_SIZE]> = MaybeUninit::uninit();

const DEFAULT_IDLE_THREAD: UnsafeCell<TcbObj> = UnsafeCell::new(TcbObj::new());
pub static IDLE_THREADS: PerCore<TcbObj, NCPU> = PerCore([DEFAULT_IDLE_THREAD; NCPU]);

global_asm!(
    r#"
.align 11
.global trap_vectors
trap_vectors:
    .align 7;
    b       unknown_exception_handler
    .align 7;
    b       unknown_exception_handler
    .align 7;
    b       unknown_exception_handler
    .align 7;
    b       unknown_exception_handler

    .align 7;
    mrs     x2, tpidr_el1;
    bic     x2, x2, #0xfff;
    mov     sp, x2;
    b       sync_handler;
    .align 7;
    mrs     x2, tpidr_el1;
    bic     x2, x2, #0xfff;
    mov     sp, x2;
    b       irq_trap;
    .align 7;
    mrs     x2, tpidr_el1;
    bic     x2, x2, #0xfff;
    mov     sp, x2;
    b       unknown_exception_handler;
    .align 7;
    mrs     x2, tpidr_el1;
    bic     x2, x2, #0xfff;
    mov     sp, x2;
    b       unknown_exception_handler;

    .align 7;
    b       lower64_trap
    .align 7;
    b       lower64_irq
    .align 7;
    b       unknown_exception_handler
    .align 7;
    b       unknown_exception_handler

    .align 7;
    b       unknown_exception_handler
    .align 7;
    b       unknown_exception_handler
    .align 7;
    b       unknown_exception_handler
    .align 7;
    b       unknown_exception_handler

lower64_trap:
    //kernel_enter
    stp     x0,  x1,  [sp, #16 * 0];
    stp     x2,  x3,  [sp, #16 * 1];
    stp     x4,  x5,  [sp, #16 * 2];
    stp     x6,  x7,  [sp, #16 * 3];
    stp     x8,  x9,  [sp, #16 * 4];
    stp     x10, x11, [sp, #16 * 5];
    stp     x12, x13, [sp, #16 * 6];
    stp     x14, x15, [sp, #16 * 7];
    stp     x16, x17, [sp, #16 * 8];
    stp     x18, x19, [sp, #16 * 9];
    stp     x20, x21, [sp, #16 * 10];
    stp     x22, x23, [sp, #16 * 11];
    stp     x24, x25, [sp, #16 * 12];
    stp     x26, x27, [sp, #16 * 13];
    stp     x28, x29, [sp, #16 * 14];
    mrs     x21, sp_el0;
    mrs     x22, elr_el1;
    mrs     x23, spsr_el1;
    stp     x30, x21, [sp, #16 * 15];
    stp     x22, x23, [sp, #16 * 16];
    mov     x0, sp
    mrs     x2, tpidr_el1
    bic     x2, x2, #0xfff
    mov     sp, x2
    b lower64_sync_handler

lower64_irq:
    //kernel_enter
    stp     x0,  x1,  [sp, #16 * 0];
    stp     x2,  x3,  [sp, #16 * 1];
    stp     x4,  x5,  [sp, #16 * 2];
    stp     x6,  x7,  [sp, #16 * 3];
    stp     x8,  x9,  [sp, #16 * 4];
    stp     x10, x11, [sp, #16 * 5];
    stp     x12, x13, [sp, #16 * 6];
    stp     x14, x15, [sp, #16 * 7];
    stp     x16, x17, [sp, #16 * 8];
    stp     x18, x19, [sp, #16 * 9];
    stp     x20, x21, [sp, #16 * 10];
    stp     x22, x23, [sp, #16 * 11];
    stp     x24, x25, [sp, #16 * 12];
    stp     x26, x27, [sp, #16 * 13];
    stp     x28, x29, [sp, #16 * 14];
    mrs     x21, sp_el0;
    mrs     x22, elr_el1;
    mrs     x23, spsr_el1;
    stp     x30, x21, [sp, #16 * 15];
    stp     x22, x23, [sp, #16 * 16];
    mov     x0, sp
    mrs     x2, tpidr_el1
    bic     x2, x2, #0xfff
    mov     sp, x2
    b lower64_irq_handler
"#
);

#[naked]
#[no_mangle]
unsafe extern "C" fn _start() {
    asm!(
        r#"
    mov     x20, x0              // save BootInfo register to x20

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
    ldr     x2, =__bss_end__
    sub     x2, x2, x1

zero_bss_loop:
    // zero out the BSS section, 64-bits at a time
    cbz     x2, jump_to_el1 
    str     xzr, [x1], #8
    sub     x2, x2, #8
    cbnz    x2, zero_bss_loop

jump_to_el1:
    /* stack pointer = kernel_stack + ((cpu_id + 1) * 4096) */
    /* x0 stored core id already */
    ldr     x1, =4096
    mul     x1, x1, x0
    ldr     x2, =kernel_stack + 4096
    add     x2, x2, x1
    mov     sp, x2

    /* Store (kernel_stack | cpu_id) in tpidr_el1 */
    orr     x0, x0, x2
    msr     tpidr_el1, x0

    /* set up exception handlers (guide: 10.4) */
    ldr     x2, =trap_vectors
    msr     VBAR_EL1, x2


    mov     x0, x20            // restore BootInfo register
    b kmain    
    "#,
        options(noreturn)
    )
}

fn initialize_init_cspace(cnode: &CNodeObj, cur_free_slot: &mut usize, bi: &BootInfo) {
    fn init_cspace_populate_untyped(
        addr_start: usize,
        addr_end: usize,
        cnode: &CNodeObj,
        cptr: &mut usize,
        is_device: bool,
    ) {
        let mut cur = addr_start;

        while cur < addr_end {
            let bit_sz = cur
                .trailing_zeros()
                .min(((addr_end - cur).next_power_of_two() / 2).trailing_zeros());
            let sz = 1 << bit_sz;

            if sz > 1 << 4 {
                kprintln!(
                    "inserting memory 0x{:x}-0x{:x} with bit size {} @cptr {} is_device {}",
                    cur,
                    cur + sz,
                    bit_sz,
                    cptr,
                    is_device
                );
                assert!(cur % sz == 0);
                cnode[*cptr].set(UntypedCap::mint(cur, bit_sz as usize, is_device));
                *cptr += 1;
            } else {
                kprintln!(
                    "skipping memory 0x{:x}-0x{:x} with bit size {}",
                    cur,
                    cur + sz,
                    bit_sz
                );
            }
            cur += sz;
        }
    }

    let kernel_base = PHYS_BASE;
    let kernel_top = (SYMBOL!(crate::_end) - KERNEL_OFFSET).next_power_of_two();
    kprintln!(
        "Kernel memory is in range 0x{:x}-0x{:x}",
        kernel_base,
        kernel_top
    );

    // /* Insert Physical memory to CSpace */
    bi.entries
        .iter()
        .filter_map(|e| match e {
            BootInfoEntry::RamEntry(r) => Some(r),
            _ => None,
        })
        .filter(|e| e.mem_type == RamType::FreeSpace)
        .for_each(|e| {
            init_cspace_populate_untyped(e.base, e.base + e.size, cnode, cur_free_slot, false);
        });

    /* Insert Init CSpace itself into it */
    let cnode_cap = CNodeCap::mint(
        cnode.as_ptr() as usize - KERNEL_OFFSET,
        cnode.len().trailing_zeros() as usize,
        64 - cnode.len().trailing_zeros() as usize,
        0,
    );
    cnode[ProcessCSpace::RootCNodeCap as usize].set(cnode_cap);

    /* Insert monitor cap for super user to control kernel */
    cnode[Monitor as usize].set(MonitorCap::mint());

    cnode[IrqController as usize].set(InterruptCap::mint());

    /* Insert Init Thread TCB */
    alloc_obj::<TcbObj>(
        &cnode,
        crate::objects::TCB_OBJ_BIT_SZ,
        &cnode[ProcessCSpace::TcbCap as usize],
    )
    .expect("Allocating Init Thread TCB failed");

    /* allocate PGD for init thread*/
    alloc_obj::<VTableObj>(&cnode, 12, &cnode[ProcessCSpace::RootVNodeCap as usize])
        .expect("Allocating PGD for Init Thread failed");
}

fn alloc_obj<'a, T>(
    cspace: &'a [CNodeEntry],
    bit_sz: usize,
    slot: &'a CNodeEntry,
) -> SysResult<CapRef<'a, T>>
where
    T: KernelObject + ?Sized,
{
    for i in UntypedStart as usize.. {
        let untyped_cap = UntypedCap::try_from(&cspace[i])?;
        let slots = core::slice::from_ref(slot);
        if let Ok(_) = untyped_cap.retype(T::obj_type, bit_sz, slots) {
            return CapRef::<T>::try_from(slot);
        }
    }

    Err(SysError::InvalidValue)
}

fn map_page_table<L: TableLevel>(tcb: &TcbObj, vaddr: usize, cur_free_slot: &mut usize) {
    let cspace = tcb.cspace().expect("Init CSpace not installed");
    let mut vspace = tcb.vspace().expect("Init VSpace not installed");
    let vaddr = VirtAddr(vaddr);

    if L::LEVEL <= Level4::LEVEL
        && !vspace
            .lookup_slot_mut::<Level4>(vaddr)
            .expect("Looking up PGD slot failed")
            .is_valid()
    {
        alloc_obj::<VTableObj>(&cspace, 12, &cspace[*cur_free_slot])
            .expect("Allocating PUD failed")
            .map_vtable(&mut vspace, vaddr, Level4::LEVEL)
            .expect("Installing PUD failed");
        *cur_free_slot += 1;
    }

    if L::LEVEL <= Level3::LEVEL
        && !vspace
            .lookup_slot::<Level3>(vaddr)
            .expect("Looking up PD slot failed")
            .is_valid()
    {
        alloc_obj::<VTableObj>(&cspace, 12, &cspace[*cur_free_slot])
            .expect("Allocating PD failed")
            .map_vtable(&mut vspace, vaddr, Level3::LEVEL)
            .expect("Installing PD failed");
        *cur_free_slot += 1;
    }

    if L::LEVEL <= Level2::LEVEL
        && !vspace
            .lookup_slot::<Level2>(vaddr)
            .expect("Looking up PT slot failed")
            .is_valid()
    {
        alloc_obj::<VTableObj>(&cspace, 12, &cspace[*cur_free_slot])
            .expect("Allocating PT failed")
            .map_vtable(&mut vspace, vaddr, Level2::LEVEL)
            .expect("Installing PT failed");
        *cur_free_slot += 1;
    }
}

fn map_frame(tcb: &TcbObj, vaddr: usize, perm: Permission, cur_free_slot: &mut usize) -> usize {
    map_page_table::<Level2>(tcb, vaddr, cur_free_slot);

    let cspace = tcb.cspace().expect("Init CSpace not installed");
    let mut vspace = tcb.vspace().expect("Init VSpace not installed");

    let frame_cap = alloc_obj::<RamObj>(&cspace, FRAME_BIT_SIZE, &cspace[*cur_free_slot])
        .expect("Allocating Frame failed");
    *cur_free_slot += 1;
    frame_cap
        .map_page::<Level1>(&mut vspace, vaddr, perm)
        .expect("Installing frame failed");
    frame_cap.vaddr()
}

pub const fn align_down(addr: usize, align: usize) -> usize {
    addr & !(align - 1)
}

struct InitThreadLoader<'a> {
    init_tcb: &'a TcbObj,
    cur_free_slot: &'a mut usize,
}

impl<'a> ElfLoader for InitThreadLoader<'a> {
    fn allocate(&mut self, load_headers: LoadableHeaders) -> Result<(), &'static str> {
        for header in load_headers {
            let flags = header.flags();
            let perm = Permission::new(flags.is_read(), flags.is_write(), flags.is_execute());
            let base = align_down(header.virtual_addr() as usize, FRAME_SIZE);
            let top = (header.virtual_addr() + header.mem_size()) as usize;
            for page_base in (base..top).step_by(FRAME_SIZE) {
                map_frame(
                    self.init_tcb,
                    page_base as usize,
                    perm,
                    &mut self.cur_free_slot,
                );
            }
        }
        Ok(())
    }

    fn relocate(&mut self, _entry: &Rela<P64>) -> Result<(), &'static str> {
        unimplemented!()
    }

    fn load(&mut self, _flags: Flags, base: VAddr, region: &[u8]) -> Result<(), &'static str> {
        let vspace = self.init_tcb.vspace().unwrap();
        let mut vaddr = align_down(base as usize, FRAME_SIZE);

        let mut region_offset = 0;
        let mut frame_offset = (base as usize) % FRAME_SIZE;

        while region_offset < region.len() {
            let frame_kvaddr: VirtAddr<KERNEL_OFFSET> = vspace
                .lookup_slot::<Level1>(VirtAddr(vaddr))
                .unwrap()
                .vaddr();
            let frame =
                unsafe { core::slice::from_raw_parts_mut(frame_kvaddr.0 as *mut u8, FRAME_SIZE) };
            let copy_len = (region.len() - region_offset).min(FRAME_SIZE) - frame_offset;
            frame[frame_offset..frame_offset + copy_len]
                .copy_from_slice(&region[region_offset..region_offset + copy_len]);

            region_offset += copy_len;
            frame_offset = (frame_offset + copy_len) % FRAME_SIZE;
            vaddr += FRAME_SIZE;
        }

        Ok(())
    }
}

fn load_init_thread(tcb: &mut TcbObj, elf_file: &[u8], cur_free_slot: &mut usize, init_fs: &[u8]) {
    let cspace = tcb.cspace().expect("Init CSpace not installed");
    let pgd_cap = VTableCap::try_from(&cspace[ProcessCSpace::RootVNodeCap as usize])
        .expect("Init PGD cap not installed");
    tcb.install_vspace(pgd_cap);

    let init_binary = ElfBinary::new("init", elf_file).expect("Invalid ELF file");
    let mut init_loader = InitThreadLoader {
        init_tcb: tcb,
        cur_free_slot,
    };
    init_binary
        .load(&mut init_loader)
        .expect("load init elf failed");

    for i in 1..PROCESS_MAIN_THREAD_STACK_PAGES + 1 {
        map_frame(
            tcb,
            PROCESS_MAIN_THREAD_STACK_TOP - i * FRAME_SIZE,
            Permission::writable(),
            cur_free_slot,
        );
    }

    let entry = init_binary.entry_point();

    let mut initfs_base = 0x40000000;
    let mut vspace = tcb.vspace().expect("Init VSpace not installed");
    for chunk in init_fs.chunks(FRAME_SIZE * 512) {
        map_page_table::<Level3>(tcb, initfs_base, cur_free_slot);
        let mut frame_obj = alloc_obj::<RamObj>(&cspace, 21, &cspace[*cur_free_slot]).unwrap();
        *cur_free_slot += 1;

        let perm = Permission::readonly();
        frame_obj
            .map_page::<Level2>(&mut vspace, initfs_base, perm)
            .expect("fail to map InitFS");
        frame_obj[0..chunk.len()].copy_from_slice(chunk);
        initfs_base += FRAME_SIZE * 512;
    }

    tcb.tf.set_elr(entry as usize);
    tcb.tf.set_sp(PROCESS_MAIN_THREAD_STACK_TOP);
    tcb.tf.init_user_thread();
}

fn run_secondary_cpus(entry: usize) {
    const SECONDARY_CPU_ENTRY_ADDR: usize = 0xe0 + KERNEL_OFFSET;

    for i in 0..crate::NCPU {
        let addr = unsafe { &mut *((SECONDARY_CPU_ENTRY_ADDR + i * 8) as *mut usize) };
        *addr = entry - KERNEL_OFFSET;
    }
}

fn init_app_cpu() {
    use crate::scheduler::SCHEDULER;

    kprintln!("Initializing Application CPU");
    IDLE_THREADS.get_mut().configure_idle_thread();
    SCHEDULER.get_mut().push(IDLE_THREADS.get());
}

fn init_bsp_cpu(bi: &BootInfo) {
    use crate::scheduler::SCHEDULER;

    crate::plat::uart::init_uart();
    kprintln!("Initializing Bootstrapping CPU");

    for (i, entry) in bi.entries.iter().enumerate() {
        if let BootInfoEntry::RamEntry(e) = entry {
            kprintln!("bi entry[{}]: {:?}", i, e);
        }
    }

    let mut cur_free_slot = UntypedStart as usize;

    let init_cnode_obj = unsafe {
        let cnode = INIT_CNODE.assume_init_mut();
        kprintln!(
            "Allocating init CNode object@{:p} size: {}",
            cnode,
            cnode.len()
        );
        for slot in cnode.iter_mut() {
            *slot = Cell::new(CapRef::<NullObj>::mint());
        }
        cnode
    };

    initialize_init_cspace(init_cnode_obj, &mut cur_free_slot, bi);
    let mut init_tcb_cap = TcbCap::try_from(&init_cnode_obj[ProcessCSpace::TcbCap as usize])
        .expect("Init TCB cap not installed");

    let init_cnode_cap =
        CNodeCap::try_from(&init_cnode_obj[ProcessCSpace::RootCNodeCap as usize]).unwrap();
    init_tcb_cap.install_cspace(&init_cnode_cap).unwrap();

    let init_fs_bytes = bi
        .entries
        .iter()
        .filter_map(|e| match e {
            BootInfoEntry::RamEntry(r) => Some(r),
            _ => None,
        })
        .find(|e| e.mem_type == RamType::InitRamFS)
        .map(|e| unsafe {
            core::slice::from_raw_parts((e.base + KERNEL_OFFSET) as *const u8, e.size)
        })
        .expect("InitFS entry not found");
    kprintln!(
        "initfs@0x{:p}-0x{:x} len {}",
        init_fs_bytes,
        init_fs_bytes.as_ptr() as usize + init_fs_bytes.len(),
        init_fs_bytes.len()
    );

    let initfs = cpio::NewcReader::from_bytes(&init_fs_bytes);
    for (i, ent) in initfs.entries().enumerate() {
        kprintln!(
            "Init fs entry[{}]: {:?}",
            i,
            core::str::from_utf8(ent.name()).unwrap()
        );
    }
    let init_thread_elf = initfs
        .entries()
        .find(|entry| entry.name() == b"init_thread")
        .map(|entry| entry.content())
        .expect("Init thread not found!");
    load_init_thread(
        &mut init_tcb_cap,
        init_thread_elf,
        &mut cur_free_slot,
        &init_fs_bytes,
    );

    run_secondary_cpus(crate::_start as usize);

    kprintln!("Jumping to User Space!");

    IDLE_THREADS.get_mut().configure_idle_thread();

    SCHEDULER.get_mut().push(IDLE_THREADS.get());
    SCHEDULER.get_mut().push(&mut init_tcb_cap);
}

#[no_mangle]
pub extern "C" fn kmain(bi_frame: usize) -> ! {
    use crate::scheduler::SCHEDULER;
    let cpuid = cpuid();

    let bi_frame =
        unsafe { BootInfo::new_from_frame((bi_frame + KERNEL_OFFSET) as *mut u8, false) };
    if cpuid == 0 {
        init_bsp_cpu(&bi_frame)
    } else {
        init_app_cpu()
    }

    unsafe {
        llvm_asm!("msr tpidrro_el0, $0"::"r"(cpuid));
    }

    let mut timer = crate::arch::generic_timer::Timer::new();
    timer.initialize(cpuid);
    timer.tick_in(crate::TICK);

    unsafe {
        vspace::mmu::flush_all_tlb();
    }
    crate::arch::clean_l1_cache();

    //TODO: somehow SCHEDULER not zeroed in bss. manually init it.
    SCHEDULER.get_mut().activate()
}
