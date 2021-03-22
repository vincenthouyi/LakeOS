use spin::Mutex;

use pi::gpio::{Gpio, Uninitialized};

use naive::space_manager::gsm;
use rustyl4api::vspace::Permission;

#[derive(Debug)]
pub struct GpioServer {
    base_addr: usize,
}

impl GpioServer {
    pub fn new(gpio_vaddr: usize) -> Self {
        Self {
            base_addr: gpio_vaddr as usize,
        }
    }

    pub fn get_pin(&mut self, pin: usize) -> Option<Gpio<Uninitialized>> {
        Some(Gpio::new(pin as u8, self.base_addr))
    }
}

pub static GPIO_SERVER: Mutex<Option<GpioServer>> = Mutex::new(None);

pub async fn init_gpio_server() {
    let gpio_ram_cap = crate::request_memory(0x3f200000, 4096, true).await.unwrap();
    let gpio_base = gsm!().insert_ram_at(gpio_ram_cap, 0, Permission::writable());

    *GPIO_SERVER.lock() = Some(GpioServer::new(gpio_base as usize));
}
