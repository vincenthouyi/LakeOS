use mutex::Mutex;

use pi::gpio::{Gpio, Uninitialized};

#[derive(Debug)]
pub struct GpioServer {
    base_addr: usize,
}

impl GpioServer {
    pub fn new(gpio_vaddr: usize) -> Self {
        Self { base_addr: gpio_vaddr as usize}
    }

    pub fn get_pin(&mut self, pin: usize) -> Option<Gpio<Uninitialized>> {
        Some(Gpio::new(pin as u8, self.base_addr))
    }
}

pub static GPIO_SERVER: Mutex<Option<GpioServer>> = Mutex::new(None);

pub fn init_gpio_server() {
    let gpio_base = naive::space_manager::allocate_frame_at(0x3f200000, 4096).unwrap();

    *GPIO_SERVER.lock() = Some(GpioServer::new(gpio_base.as_ptr() as usize));
}
