use crate::process::ProcessCSpace;

#[repr(usize)]
pub enum InitCSpaceSlot {
    Monitor = ProcessCSpace::WellKnownMax as usize,
    IrqController,

    UntypedStart,
}
