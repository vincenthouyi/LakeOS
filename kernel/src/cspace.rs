use crate::objects::{CNodeObj, CNodeEntry, CNodeLookupErr};

pub struct CSpace<'a>(pub &'a mut CNodeObj);

impl<'a> core::ops::Deref for CSpace<'a> {
    type Target = CNodeObj;
    fn deref(&self) -> &Self::Target {
        self.0
    }
}

impl<'a> CSpace<'a> {
    pub fn lookup_slot(&self, idx: usize) -> Result<&CNodeEntry, CNodeLookupErr> {
        let slot = self.0.get(idx).ok_or(CNodeLookupErr::CNodeMiss(idx))?;
        Ok(unsafe { &*(slot as *const CNodeEntry) })
    }
}

impl<'a> core::ops::DerefMut for CSpace<'a> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.0
    }
}