pub const PROCESS_ROOT_CNODE_SIZE: usize = 2048;
pub const PROCESS_MAIN_THREAD_STACK_TOP: usize = 0x8000000;
pub const PROCESS_MAIN_THREAD_STACK_PAGES: usize = 4;

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
    NameServer,
    WellKnownMax,
}
