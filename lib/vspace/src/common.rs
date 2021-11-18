pub trait Level {
    const LEVEL: usize;
}

pub trait TableLevel: Level {
    type NextLevel: Level;
}

pub trait PageLevel: Level {
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
#[derive(Copy, Clone, Debug)]
pub enum Level0 {}

impl Level for Level4 {
    const LEVEL: usize = 4;
}
impl Level for Level3 {
    const LEVEL: usize = 3;
}
impl Level for Level2 {
    const LEVEL: usize = 2;
}
impl Level for Level1 {
    const LEVEL: usize = 1;
}
impl Level for Level0 {
    const LEVEL: usize = 1;
}

impl TableLevel for Level4 {
    type NextLevel = Level3;
}
impl TableLevel for Level3 {
    type NextLevel = Level2;
}
impl TableLevel for Level2 {
    type NextLevel = Level1;
}
impl TableLevel for Level1 {
    type NextLevel = Level0;
}
