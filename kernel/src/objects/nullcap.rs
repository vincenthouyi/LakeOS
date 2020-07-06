use super::*;

#[derive(Debug)]
pub enum NullObj {}

pub type NullCap<'a> = CapRef<'a, NullObj>;

impl<'a> CapRef<'a, NullObj> {
    pub const fn mint() -> CapRaw {
        CapRaw::new(0, 0, 0, None, None, ObjType::NullObj)
    }

    pub fn insert<T>(self, raw: CapRaw) -> CapRef<'a, T> 
        where T: KernelObject + ?Sized
    {
        debug_assert_eq!(T::obj_type(), raw.cap_type());
        self.raw.set(raw);

        CapRef {
            raw: self.raw,
            cap_type: PhantomData
        }
    }

    pub fn debug_formatter(_f: &mut core::fmt::DebugStruct, _cap: &CapRaw) {
        return;
    }

    pub fn identify(&self, tcb: &TcbObj) -> usize {
        tcb.set_mr(1, self.cap_type() as usize);
        1
    }
}
