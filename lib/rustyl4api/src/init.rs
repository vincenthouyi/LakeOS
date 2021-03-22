use crate::process::ProcessCSpace;

#[repr(usize)]
pub enum InitCSpaceSlot {
    Monitor = ProcessCSpace::WellKnownMax as usize,
    IrqController,

    UntypedStart,
}

pub const INIT_STACK_PAGES: usize = 4;
pub const INIT_STACK_TOP: usize = 0x600000;
