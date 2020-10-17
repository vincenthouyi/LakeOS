use super::*;

#[derive(Debug)]
pub enum MonitorObj {}

pub type MonitorCap<'a> = CapRef<'a, MonitorObj>;

impl<'a> MonitorCap<'a> {
    pub const fn mint() -> CapRaw {
        CapRaw::new(0, 0, 0, None, None, ObjType::Monitor)
    }

    pub fn debug_formatter(_f: &mut core::fmt::DebugStruct, _cap: &CapRaw) {
        return;
    }

    pub fn identify(&self, tcb: &mut TcbObj) -> usize {
        tcb.set_mr(1, self.cap_type() as usize);
        1
    }
}
