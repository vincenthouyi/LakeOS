use num_traits::FromPrimitive;

#[derive(Copy, Clone, Debug)]
pub enum Fault {
    DataFault(VmFaultInfo),
    PrefetchFault(VmFaultInfo),
}

impl Fault {
    pub fn new_data_fault(addr: u64, level: u8, kind: VmFaultKind) -> Self {
        Self::DataFault(VmFaultInfo {
            address: addr,
            level,
            kind
        })
    }

    pub fn new_prefetch_fault(addr: u64, level: u8, kind: VmFaultKind) -> Self {
        Self::DataFault(VmFaultInfo {
            address: addr,
            level,
            kind
        })
    }

    pub fn as_ipc_message_buf(&self) -> [usize; 3] {
        let mut buf = [0; 3];
        match self {
            Self::DataFault(info) => {
                buf[0] = 0;
                buf[1] = info.address as usize;
                buf[2] = (info.level as usize) << 32 | info.kind as usize;
            }
            Self::PrefetchFault(info) => {
                buf[0] = 1;
                buf[1] = info.address as usize;
                buf[2] = (info.level as usize) << 32 | info.kind as usize;
            }
        }
        buf
    }

    pub fn from_ipc_message_buf(buf: &[usize]) -> Self {
        let addr = buf[1] as u64;
        let level = (buf[2] >> 32) as u8;
        let kind = buf[2] as u8;
        let info = VmFaultInfo {
            address: addr,
            level: level,
            kind: VmFaultKind::from_u8(kind).unwrap(),
        };
        if buf[0] == 0 {
            Self::DataFault(info)
        } else if buf[0] == 1 {
            Self::PrefetchFault(info)
        } else {
            panic!()
        }
    }
}

#[derive(Copy, Clone, Debug)]
pub struct VmFaultInfo {
    pub address: u64,
    pub level: u8,
    pub kind: VmFaultKind,
}

#[repr(u8)]
#[derive(Copy, Clone, Debug, FromPrimitive)]
pub enum VmFaultKind {
    AddressSize = 0,
    Translation = 1,
    AccessFlag = 2,
    Permission = 3,
    Alignment = 4,
    TlbConflict = 5,
    Other = 6,
}