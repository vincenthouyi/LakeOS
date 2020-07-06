use super::*;

#[derive(Debug)]
pub enum InterruptObj {}

pub type InterruptCap<'a> = CapRef<'a, InterruptObj>;

impl<'a> InterruptCap<'a> {
    pub const fn mint() -> CapRaw {
        CapRaw::new(0, 0, 0, None, None, ObjType::Interrupt)
    }

    pub fn debug_formatter(_f: &mut core::fmt::DebugStruct, _cap: &CapRaw) {
        return;
    }

    pub fn identify(&self, tcb: &TcbObj) -> usize {
        tcb.set_mr(1, self.cap_type() as usize);
        1
    }
}
