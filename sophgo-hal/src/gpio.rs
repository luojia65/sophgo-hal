//! General Purpose Input/Output.
use crate::{GPIO, PADS};
use base_address::BaseAddress;
use core::marker::PhantomData;
use volatile_register::{RO, RW, WO};

/// GPIO registers.
#[repr(C)]
pub struct RegisterBlock {
    /// Port A data register.
    pub data: RW<u32>,
    /// Port A data direction register.
    pub direction: RW<Direction>,
    _reserved0: [u8; 0x28],
    /// Interrupt enable register.
    pub interrupt_enable: RW<u32>,
    /// Interrupt mask register.
    pub interrupt_mask: RW<u32>,
    /// Interrupt level register.
    pub interrupt_level: RW<u32>,
    /// Interrupt polarity register.
    pub interrupt_polarity: RW<u32>,
    /// Interrupt status register.
    pub interrupt_status: RO<u32>,
    /// Raw interrupt status register.
    pub raw_interrupt_status: RO<u32>,
    /// Debounce enable register.
    pub debounce: RW<u32>,
    /// Port A clear interrupt register.
    pub interrupt_clear: WO<u32>,
    /// Port A external port register.
    pub external_port: RW<u32>,
    _reserved1: [u8; 0xC],
    /// Level-sensitive synchronization enable register.
    pub sync_level: RW<u32>,
}

/// GPIO direction.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
#[repr(transparent)]
pub struct Direction(u32);

impl Direction {
    /// Set GPIO direction to input.
    #[inline]
    pub fn set_input(self, n: usize) -> Self {
        Self(self.0 & !(1 << n))
    }
    /// Set GPIO direction to output.
    #[inline]
    pub fn set_output(self, n: usize) -> Self {
        Self(self.0 | (1 << n))
    }
    /// Check if GPIO direction is input.
    #[inline]
    pub fn is_input(self, n: usize) -> bool {
        self.0 & (1 << n) == 0
    }
    /// Check if GPIO direction is output.
    #[inline]
    pub fn is_output(self, n: usize) -> bool {
        self.0 & (1 << n) != 0
    }
}

pub struct Pad<A: BaseAddress, const N: usize, M> {
    base: PADS<A>,
    _function: PhantomData<M>,
}

pub struct Gpio<A1: BaseAddress, A2: BaseAddress, const N: usize, M> {
    base: PADS<A1>,
    gpio: GPIO<A2>,
    _function: PhantomData<M>,
}

impl<A: BaseAddress, const N: usize, M> Pad<A, N, M> {
    #[inline]
    pub fn into_alternate<const M2: u32>(self) -> Pad<A, N, Function<M2>> {
        unsafe { self.base.fmux::<N>().write(M2) };
        Pad {
            base: self.base,
            _function: PhantomData,
        }
    }
}

impl<A1: BaseAddress, A2: BaseAddress, const N: usize, M> Gpio<A1, A2, N, M>
where
    PadNum<N>: HasGpioMode,
{
    #[inline]
    pub fn into_pull_up_output(self, gpio: &GPIO<A2>) -> Gpio<A1, A2, N, Output<PullUp>> {
        unsafe {
            self.base.fmux::<N>().write(PadNum::<N>::GPIO_MODE);
            self.base.pad_config::<N>().modify(|cfg| cfg); // TODO set_pull(Pull::Up)
            self.gpio.direction.modify(|val| val.set_output(N));
        }
        Pad {
            base: self.base,
            gpio: self.gpio,
            _function: PhantomData,
        }
    }
}

impl<A1: BaseAddress, A2: BaseAddress, const N: usize, M> embedded_hal::digital::ErrorType
    for Gpio<A1, A2, N, Output<M>>
{
    type Error = core::convert::Infallible;
}

impl<A1: BaseAddress, A2: BaseAddress, const N: usize, M> embedded_hal::digital::OutputPin
    for Gpio<A1, A2, N, Output<M>>
{
    #[inline]
    fn set_low(&mut self) -> Result<(), Self::Error> {
        unsafe { self.gpio.data.modify(|val| val & !(1 << N)) };
        Ok(())
    }

    #[inline]
    fn set_high(&mut self) -> Result<(), Self::Error> {
        unsafe { self.gpio.data.modify(|val| val | (1 << N)) };
        Ok(())
    }
}

pub struct Output<M> {
    _mode: PhantomData<M>,
}

// TODO Input<M>

pub struct PullUp;

// TODO Floating, PullDown

pub struct PadNum<const N: usize>;

pub struct Function<const M: u32>;

pub trait HasGpioMode {
    const GPIO_MODE: u32;
}

#[rustfmt::skip]
mod has_gpio_mode_impls {
    use super::{HasGpioMode, PadNum};
    impl HasGpioMode for PadNum<3> { const GPIO_MODE: u32 = 3; }
    impl HasGpioMode for PadNum<49> { const GPIO_MODE: u32 = 0; }
}

pub struct Pads<A1: BaseAddress, A2: BaseAddress> {
    pub sd0_clk: Pad<A1, A2, 3, Function<0>>,
    pub sd0_cmd: Pad<A1, A2, 4, Function<0>>,
}

pub struct PowerPads<A1: BaseAddress, A2: BaseAddress> {
    pub pwr_gpio2: Pad<A1, A2, 49, Function<0>>,
}

#[cfg(test)]
mod tests {
    use super::RegisterBlock;
    use memoffset::offset_of;

    #[test]
    fn struct_register_block_offset() {
        assert_eq!(offset_of!(RegisterBlock, data), 0x00);
        assert_eq!(offset_of!(RegisterBlock, direction), 0x04);
        assert_eq!(offset_of!(RegisterBlock, interrupt_enable), 0x30);
        assert_eq!(offset_of!(RegisterBlock, interrupt_mask), 0x34);
        assert_eq!(offset_of!(RegisterBlock, interrupt_level), 0x38);
        assert_eq!(offset_of!(RegisterBlock, interrupt_polarity), 0x3C);
        assert_eq!(offset_of!(RegisterBlock, interrupt_status), 0x40);
        assert_eq!(offset_of!(RegisterBlock, raw_interrupt_status), 0x44);
        assert_eq!(offset_of!(RegisterBlock, debounce), 0x48);
        assert_eq!(offset_of!(RegisterBlock, interrupt_clear), 0x4C);
        assert_eq!(offset_of!(RegisterBlock, external_port), 0x50);
        assert_eq!(offset_of!(RegisterBlock, sync_level), 0x60);
    }
}
