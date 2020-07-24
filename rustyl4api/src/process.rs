
pub const PROCESS_ROOT_CNODE_SIZE: usize = 1024;

#[repr(usize)]
pub enum ProcessCSpace {
    NullCap = 0,
    TcbCap,
    RootCNodeCap,
    RootVNodeCap,
    InitUntyped,
    Stdin,
    Stdout,
    Stderr,
    ProcessFixedMax,
}