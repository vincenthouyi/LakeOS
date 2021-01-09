#[repr(C)]
pub enum InitCSpaceSlot {
    NullCap = 0,
    InitTCB,
    InitCSpace,
    InitL1PageTable,
    Monitor,
    IrqController,

    UntypedStart,
}

pub const INIT_CSPACE_SIZE: usize = 1024;
pub const INIT_STACK_PAGES: usize = 4;
pub const INIT_STACK_TOP: usize = 0x600000;
