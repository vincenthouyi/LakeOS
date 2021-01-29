use crate::prelude::*;
mod cnode;
mod endpoint;
mod interrupt;
mod monitor;
mod nullcap;
mod ram;
mod reply;
mod tcb;
mod traits;
mod untyped;
mod vtable;

use core::cell::Cell;
use core::marker::PhantomData;
use core::ptr::NonNull;

pub use cnode::*;
pub use endpoint::*;
pub use interrupt::*;
pub use monitor::*;
pub use nullcap::*;
pub use ram::*;
pub use reply::*;
pub use sysapi::object::ObjType;
pub use tcb::*;
pub use traits::*;
pub use untyped::*;
pub use vtable::*;

/* Capability Entry Field Definition
 * -----------------------------------------------
 * |                 prev                   |type|
 * |                  59                    | 5  |
 * -----------------------------------------------
 * |                 next                   |    |
 * |                  59                    |    |
 * -----------------------------------------------
 */
#[derive(Debug, Clone, Copy)]
pub struct CapRef<'a, T: KernelObject + ?Sized> {
    raw: &'a CNodeEntry,
    cap_type: PhantomData<T>,
}

impl<'a, T: KernelObject + ?Sized> CapRef<'a, T> {
    pub fn cap_type(&self) -> ObjType {
        debug_assert_eq!(T::obj_type, self.raw().cap_type());
        T::obj_type
    }

    pub fn raw(&self) -> CapRaw {
        self.raw.get()
    }

    pub fn paddr(&self) -> usize {
        self.raw.get().paddr
    }

    pub fn vaddr(&self) -> usize {
        self.paddr() + KERNEL_OFFSET
    }

    fn _retype<U: KernelObject + ?Sized>(self) -> CapRef<'a, U> {
        debug_assert_eq!(U::obj_type, self.raw().cap_type());
        CapRef {
            raw: self.raw,
            cap_type: PhantomData,
        }
    }

    pub fn take(self) -> (NullCap<'a>, CapRaw) {
        let raw = self.raw.take();
        (self._retype(), raw)
    }

    pub fn append_next(&self, cap: &CNodeEntry) {
        let mut self_raw = self.raw();
        let mut cap_raw = cap.get();
        let orig_next = self_raw.get_next();
        orig_next.map(|next_ptr| {
            let next_cap = unsafe { next_ptr.as_ref() };
            let mut next_raw = next_cap.get();
            next_raw.set_prev(Some(NonNull::from(cap)));
            next_cap.set(next_raw);
        });
        cap_raw.set_next(orig_next);
        cap_raw.set_prev(Some(NonNull::from(self.raw)));
        cap.set(cap_raw);
        self_raw.set_next(Some(NonNull::from(cap)));
        self.raw.set(self_raw);
    }
}

impl<'a, T: KernelObject + Sized> CapRef<'a, T> {
    fn obj_ptr(&self) -> NonNull<T> {
        NonNull::new(self.vaddr() as *mut T).unwrap()
    }

    pub unsafe fn get_obj_mut(&self) -> &mut T {
        &mut *self.obj_ptr().as_ptr()
    }
}

impl<'a, T: KernelObject + Default> CapRef<'a, T> {
    pub fn init(&mut self) {
        let obj: &mut T = self;
        *obj = T::default();
    }
}

impl<'a, T: KernelObject + Sized> core::ops::Deref for CapRef<'a, T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        unsafe { &*self.obj_ptr().as_ptr() }
    }
}

impl<'a, T: KernelObject + Sized> core::ops::DerefMut for CapRef<'a, T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        unsafe { &mut *self.obj_ptr().as_ptr() }
    }
}

impl<'a, T: ?Sized + KernelObject> core::convert::TryFrom<&'a CNodeEntry> for CapRef<'a, T> {
    type Error = SysError;

    fn try_from(value: &'a CNodeEntry) -> SysResult<Self> {
        if T::obj_type != value.get().cap_type() {
            Err(Self::Error::CapabilityTypeError)
        } else {
            Ok(Self {
                raw: value,
                cap_type: PhantomData,
            })
        }
    }
}

#[repr(align(32))]
#[derive(Clone, Copy, Default, PartialEq)]
pub struct CapRaw {
    arg1: usize,
    arg2: usize,
    pub paddr: usize,
    pub cap_type: ObjType,
    pub prev: Option<NonNull<CNodeEntry>>,
    pub next: Option<NonNull<CNodeEntry>>,
}

impl CapRaw {
    pub const fn new(
        paddr: usize,
        arg1: usize,
        arg2: usize,
        prev: Option<NonNull<CNodeEntry>>,
        next: Option<NonNull<CNodeEntry>>,
        cap_type: ObjType,
    ) -> Self {
        Self {
            arg1: arg1,
            arg2: arg2,
            paddr: paddr,
            cap_type: cap_type,
            prev: prev,
            next: next,
        }
    }

    pub fn cap_type(&self) -> ObjType {
        self.cap_type
    }

    fn set_prev(&mut self, prev: Option<NonNull<CNodeEntry>>) {
        self.prev = prev;
    }

    fn set_next(&mut self, next: Option<NonNull<CNodeEntry>>) {
        self.next = next;
    }

    pub fn get_prev(&self) -> Option<NonNull<CNodeEntry>> {
        self.prev
    }

    pub fn get_next(&self) -> Option<NonNull<CNodeEntry>> {
        self.next
    }
}

impl core::fmt::Debug for CapRaw {
    fn fmt(&self, f: &mut core::fmt::Formatter) -> Result<(), core::fmt::Error> {
        let mut formatter = f.debug_struct("CapRaw");
        formatter.field("cap type", &self.cap_type());
        match self.cap_type() {
            ObjType::NullObj => CapRef::<NullObj>::debug_formatter(&mut formatter, self),
            ObjType::Untyped => CapRef::<UntypedObj>::debug_formatter(&mut formatter, self),
            ObjType::CNode => CapRef::<CNodeObj>::debug_formatter(&mut formatter, self),
            ObjType::Tcb => CapRef::<TcbObj>::debug_formatter(&mut formatter, self),
            ObjType::Ram => CapRef::<RamObj>::debug_formatter(&mut formatter, self),
            ObjType::VTable => CapRef::<VTableObj>::debug_formatter(&mut formatter, self),
            ObjType::Endpoint => CapRef::<EndpointObj>::debug_formatter(&mut formatter, self),
            ObjType::Reply => CapRef::<ReplyObj>::debug_formatter(&mut formatter, self),
            ObjType::Monitor => CapRef::<MonitorObj>::debug_formatter(&mut formatter, self),
            ObjType::Interrupt => CapRef::<InterruptObj>::debug_formatter(&mut formatter, self),
        }
        formatter.field("prev", &self.get_prev());
        formatter.field("next", &self.get_next());
        formatter.finish()
    }
}
