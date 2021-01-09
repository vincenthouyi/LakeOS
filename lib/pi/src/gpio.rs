use core::marker::PhantomData;

use common::states;
use volatile::Volatile;

/// An alternative GPIO function.
#[repr(u8)]
pub enum Function {
    Input = 0b000,
    Output = 0b001,
    Alt0 = 0b100,
    Alt1 = 0b101,
    Alt2 = 0b110,
    Alt3 = 0b111,
    Alt4 = 0b011,
    Alt5 = 0b010,
}

#[repr(C)]
#[allow(non_snake_case)]
struct Registers {
    FSEL: [u32; 6],
    __r0: u32,
    SET: [u32; 2],
    __r1: u32,
    CLR: [u32; 2],
    __r2: u32,
    LEV: [u32; 2],
    __r3: u32,
    EDS: [u32; 2],
    __r4: u32,
    REN: [u32; 2],
    __r5: u32,
    FEN: [u32; 2],
    __r6: u32,
    HEN: [u32; 2],
    __r7: u32,
    LEN: [u32; 2],
    __r8: u32,
    AREN: [u32; 2],
    __r9: u32,
    AFEN: [u32; 2],
    __r10: u32,
    PUD: u32,
    PUDCLK: [u32; 2],
}

// Possible states for a GPIO pin.
states! {
    Uninitialized, Input, Output, Alt
}

/// A GPIO pin in state `State`.
///
/// The `State` generic always corresponds to an uninstantiatable type that is
/// use solely to mark and track the state of a given GPIO pin. A `Gpio`
/// structure starts in the `Uninitialized` state and must be transitions into
/// one of `Input`, `Output`, or `Alt` via the `into_input`, `into_output`, and
/// `into_alt` methods before it can be used.
pub struct Gpio<State> {
    pin: u8,
    registers: &'static mut Registers,
    _state: PhantomData<State>,
}

/// The base address of the `GPIO` registers.
//const GPIO_BASE: usize = IO_BASE + 0x200000;

impl<T> Gpio<T> {
    /// Transitions `self` to state `S`, consuming `self` and returning a new
    /// `Gpio` instance in state `S`. This method should _never_ be exposed to
    /// the public!
    #[inline(always)]
    fn transition<S>(self) -> Gpio<S> {
        Gpio {
            pin: self.pin,
            registers: self.registers,
            _state: PhantomData,
        }
    }
}

impl Gpio<Uninitialized> {
    /// Returns a new `GPIO` structure for pin number `pin`.
    ///
    /// # Panics
    ///
    /// Panics if `pin` > `53`.
    pub fn new(pin: u8, gpio_base: usize) -> Gpio<Uninitialized> {
        if pin > 53 {
            panic!("Gpio::new(): pin {} exceeds maximum of 53", pin);
        }

        Gpio {
            registers: unsafe { &mut *(gpio_base as *mut Registers) },
            pin: pin,
            _state: PhantomData,
        }
    }

    /// Enables the alternative function `function` for `self`. Consumes self
    /// and returns a `Gpio` structure in the `Alt` state.
    pub fn into_alt(self, function: Function) -> Gpio<Alt> {
        let reg = self.pin as usize / 10;
        let bit = (self.pin as usize % 10) * 3;

        let val = Volatile::new_read_only(&self.registers.FSEL[reg]).read() & !(0b111 << bit);
        let val = val | (function as u32) << bit;
        Volatile::new_write_only(&mut self.registers.FSEL[reg]).write(val);

        self.transition()
    }

    /// Sets this pin to be an _output_ pin. Consumes self and returns a `Gpio`
    /// structure in the `Output` state.
    pub fn into_output(self) -> Gpio<Output> {
        self.into_alt(Function::Output).transition()
    }

    /// Sets this pin to be an _input_ pin. Consumes self and returns a `Gpio`
    /// structure in the `Input` state.
    pub fn into_input(self) -> Gpio<Input> {
        self.into_alt(Function::Input).transition()
    }
}

impl Gpio<Output> {
    /// Sets (turns on) the pin.
    pub fn set(&mut self) {
        let reg = self.pin as usize / 32;
        let bit = self.pin as usize % 32;
        Volatile::new_write_only(&mut self.registers.SET[reg]).write(1 << bit)
    }

    /// Clears (turns off) the pin.
    pub fn clear(&mut self) {
        let reg = self.pin as usize / 32;
        let bit = self.pin as usize % 32;
        Volatile::new_write_only(&mut self.registers.CLR[reg]).write(1 << bit)
    }
}

impl Gpio<Input> {
    /// Reads the pin's value. Returns `true` if the level is high and `false`
    /// if the level is low.
    pub fn level(&mut self) -> bool {
        let reg = self.pin as usize / 32;
        let bit = self.pin as usize % 32;
        let val = Volatile::new_read_only(&self.registers.LEV[reg]).read() & (1 << bit);
        match val {
            0 => false,
            _ => true,
        }
    }
}
