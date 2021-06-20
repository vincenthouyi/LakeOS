use crate::common::*;

pub trait PageLevel: TableLevel {
    const FRAME_BIT_SIZE: usize;
}
impl PageLevel for Level3 {
    const FRAME_BIT_SIZE: usize = 30;
}
impl PageLevel for Level2 {
    const FRAME_BIT_SIZE: usize = 21;
}
impl PageLevel for Level1 {
    const FRAME_BIT_SIZE: usize = 12;
}
