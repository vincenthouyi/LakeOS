use core::fmt;

use volatile::prelude::*;
use volatile::{Volatile, ReadVolatile, Reserved};

/// The base address for the `MU` registers.
//const MU_REG_PAGE_BASE: usize = IO_BASE + 0x215000;
//const MU_REG_BASE: usize = MU_REG_PAGE_BASE + MU_REG_OFFSET;

/// The `AUXENB` register from page 9 of the BCM2837 documentation.
//const AUX_ENABLES: *mut Volatile<u8> = (MU_REG_PAGE_BASE + AUX_ENABLES_OFFSET) as *mut Volatile<u8>;

const MU_REG_OFFSET: usize = 0x40;
const AUX_ENABLES_OFFSET: usize = 0x4;

/// Enum representing bit fields of the `AUX_MU_LSR_REG` register.
#[repr(u8)]
enum LsrStatus {
    DataReady = 1,
    TxAvailable = 1 << 5,
}

enum IrqBits {
    Rx = 0b01,
    Tx = 0b10,
}

pub enum IrqStatus {
    Clear,
    Tx,
    Rx,
}

#[repr(C)]
#[allow(non_snake_case)]
struct Registers {
    AUX_MU_IO_REG: Volatile<u8>, /* 0x7E21 5040 */
    __r0: [Reserved<u8>; 3],
    AUX_MU_IER_REG: Volatile<u8>,
    __r1: [Reserved<u8>; 3],
    AUX_MU_IIR_REG: Volatile<u8>,
    __r2: [Reserved<u8>; 3],
    AUX_MU_LCR_REG: Volatile<u8>,
    __r3: [Reserved<u8>; 3],
    AUX_MU_MCR_REG: Volatile<u8>,
    __r4: [Reserved<u8>; 3],
    AUX_MU_LSR_REG: ReadVolatile<u8>,
    __r5: [Reserved<u8>; 3],
    AUX_MU_MSR_REG: ReadVolatile<u8>,
    __r6: [Reserved<u8>; 3],
    AUX_MU_SCRATCH: Volatile<u8>,
    __r7: [Reserved<u8>; 3],
    AUX_MU_CNTL_REG: Volatile<u8>,
    __r8: [Reserved<u8>; 3],
    AUX_MU_STAT_REG: ReadVolatile<u32>,
    AUX_MU_BAUD_REG: Volatile<u16>,
    __r9: [Reserved<u8>; 22], /* 0x7E21 506A */
    /* AUX_SPI0 */
    AUX_SPI0_CNTL0_REG: Volatile<u32>, /* 0x7E21 5080 */
    AUX_SPI0_CNTL1_REG: Volatile<u8>,
    __r10: [Reserved<u8>; 3],
    AUX_SPI0_STAT_REG: Volatile<u32>,
    __r11: [Reserved<u8>; 4],
    AUX_SPI0_IO_REG: Volatile<u32>,
    AUX_SPI0_PEEK_REG: Volatile<u16>,
    __r12: [Reserved<u8>; 42], /* 0x7E21 5096 */
    /* AUX_SPI1 */
    AUX_SPI1_CNTL0_REG: Volatile<u32>, /* 0x7E21 50C0 */
    AUX_SPI1_CNTL1_REG: Volatile<u8>,
    __r13: [Reserved<u8>; 3],
    AUX_SPI1_STAT_REG: Volatile<u32>,
    __r14: [Reserved<u8>; 4],
    AUX_SPI1_IO_REG: Volatile<u32>,
    AUX_SPI1_PEEK_REG: Volatile<u16>,
}

/// The Raspberry Pi's "mini UART".
pub struct MiniUart {
    page_base: usize,
    registers: &'static mut Registers,
}

impl MiniUart {
    /// Initializes the mini UART by enabling it as an auxiliary peripheral,
    /// setting the data size to 8 bits, setting the BAUD rate to ~115200 (baud
    /// divider of 270), setting GPIO pins 14 and 15 to alternative function 5
    /// (TXD1/RDXD1), and finally enabling the UART transmitter and receiver.
    ///
    /// By default, reads will never time out. To set a read timeout, use
    /// `set_read_timeout()`.
    pub fn new(page_base: usize) -> MiniUart {
        let registers = unsafe {
            &mut *((page_base + MU_REG_OFFSET)as *mut Registers)
        };

        MiniUart {
            page_base: page_base,
            registers: registers,
        }
    }

    pub fn initialize(&mut self, _baud_rate: usize) {
        let aux_enables = (self.page_base + AUX_ENABLES_OFFSET) as *mut Volatile<u8>;
        unsafe {
            (*aux_enables).or_mask(1);
        }
        self.registers.AUX_MU_LCR_REG.or_mask(0b1); // Set in 8-bit mode
        self.registers.AUX_MU_BAUD_REG.write(270); // Set baudrate
        self.registers.AUX_MU_CNTL_REG.or_mask(0b11); // Enable Rx Tx
    }

    pub fn can_write(&self) -> bool {
        self.registers.AUX_MU_LSR_REG.has_mask(LsrStatus::TxAvailable as u8)
    }

    /// Write the byte `byte`. This method blocks until there is space available
    /// in the output FIFO.
    pub fn write_byte(&mut self, byte: u8) {
        while !self.can_write() { };
        self.registers.AUX_MU_IO_REG.write(byte);
    }

    /// Returns `true` if there is at least one byte ready to be read. If this
    /// method returns `true`, a subsequent call to `read_byte` is guaranteed to
    /// return immediately. This method does not block.
    pub fn has_byte(&self) -> bool {
        self.registers.AUX_MU_LSR_REG.has_mask(LsrStatus::DataReady as u8)
    }

    /// Reads a byte. Blocks indefinitely until a byte is ready to be read.
    pub fn read_byte(&mut self) -> u8 {
        while !self.has_byte() { };
        self.registers.AUX_MU_IO_REG.read()
    }

    fn enable_irq(&mut self, irq: IrqBits) {
        self.registers.AUX_MU_IER_REG.or_mask(irq as u8);
    }

    fn disable_irq(&mut self, irq: IrqBits) {
        self.registers.AUX_MU_IER_REG.and_mask(!(irq as u8));
    }

    pub fn enable_tx_irq(&mut self) {
        self.enable_irq(IrqBits::Tx)
    }

    pub fn disable_tx_irq(&mut self) {
        self.disable_irq(IrqBits::Tx)
    }

    pub fn enable_rx_irq(&mut self) {
        self.enable_irq(IrqBits::Rx)
    }

    pub fn disable_rx_irq(&mut self) {
        self.disable_irq(IrqBits::Rx)
    }

    pub fn irq_status(&self) -> IrqStatus {
        match (self.registers.AUX_MU_IIR_REG.read() >> 1) & 0b11 {
            0b00 => { IrqStatus::Clear }
            0b01 => { IrqStatus::Tx }
            0b10 => { IrqStatus::Rx }
            _ => { unreachable!() }
        }
    }
}

// FIXME: Implement `fmt::Write` for `MiniUart`. A b'\r' byte should be written
// before writing any b'\n' byte.
impl fmt::Write for MiniUart {
    fn write_str(&mut self, s: &str) -> fmt::Result {
        for byte in s.bytes() {
            if byte == b'\n' {
                self.write_byte(b'\r');
            }
            self.write_byte(byte);
        }
        Ok(())
    }
}
