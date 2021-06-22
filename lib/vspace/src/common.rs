
pub trait TableLevel {
    const LEVEL: usize;
    type NextLevel: TableLevel;
}

pub trait PageLevel: TableLevel {
    const FRAME_BIT_SIZE: usize;
}

#[derive(Copy, Clone, Debug)]
pub enum Level4 {}
#[derive(Copy, Clone, Debug)]
pub enum Level3 {}
#[derive(Copy, Clone, Debug)]
pub enum Level2 {}
#[derive(Copy, Clone, Debug)]
pub enum Level1 {}

impl TableLevel for Level4 {
    const LEVEL: usize = 4;
    type NextLevel = Level3;
}
impl TableLevel for Level3 {
    const LEVEL: usize = 3;
    type NextLevel = Level2;
}
impl TableLevel for Level2 {
    const LEVEL: usize = 2;
    type NextLevel = Level1;
}
impl TableLevel for Level1 {
    const LEVEL: usize = 1;
    type NextLevel = Level4;
}
