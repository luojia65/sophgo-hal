use crate::UART;
use base_address::BaseAddress;
use embedded_time::rate::Baud;
use uart16550::LineControl;

pub struct Serial<A: BaseAddress, const U: usize, PADS> {
    base: UART<A>,
    pads: PADS,
}

impl<A: BaseAddress, const U: usize, PADS> Serial<A, U, PADS> {
    /// Creates a polling serial instance, without interrupt or DMA configurations.
    #[inline]
    pub fn new(base: UART<A>, config: Config, baudrate: Baud, pads: PADS) -> Self
    where
        PADS: Pads<U>,
    {
        let _ = baudrate; // TODO configure baudrate
        let parity = match config.parity {
            Parity::Even => uart16550::PARITY::EVEN,
            Parity::Odd => uart16550::PARITY::ODD,
            Parity::None => uart16550::PARITY::NONE,
        };
        let one_stop_bit = match config.stop_bits {
            StopBits::One => true,
            StopBits::Two => false,
        };
        let word_length = match config.word_length {
            WordLength::Five => uart16550::CharLen::FIVE,
            WordLength::Six => uart16550::CharLen::SIX,
            WordLength::Seven => uart16550::CharLen::SEVEN,
            WordLength::Eight => uart16550::CharLen::EIGHT,
        };
        base.lcr().write(
            LineControl::CONFIG_8N1
                .set_parity(parity)
                .set_char_len(word_length)
                .set_one_stop_bit(one_stop_bit),
        );
        Self { base, pads }
    }

    /// Release serial instance and return its peripheral and pads.
    #[inline]
    pub fn free(self) -> (UART<A>, PADS) {
        (self.base, self.pads)
    }
}

/// Valid UART pads.
pub trait Pads<const U: usize> {
    /// Checks if this pad configuration includes Request-to-Send feature.
    const RTS: bool;
    /// Checks if this pad configuration includes Clear-to-Send feature.
    const CTS: bool;
    /// Checks if this pad configuration includes Transmit feature.
    const TXD: bool;
    /// Checks if this pad configuration includes Receive feature.
    const RXD: bool;
}

impl<A: BaseAddress, const U: usize, PADS> embedded_io::ErrorType for Serial<A, U, PADS> {
    type Error = core::convert::Infallible; // TODO
}

impl<A: BaseAddress, const U: usize, PADS> embedded_io::Write for Serial<A, U, PADS> {
    #[inline]
    fn write(&mut self, buf: &[u8]) -> Result<usize, Self::Error> {
        // TODO handle error
        let mut count = 0usize;
        for byte in buf {
            if self.base.lsr().read().is_transmitter_fifo_empty() {
                self.base.rbr_thr().tx_data(*byte);
                count += 1;
            } else {
                break;
            }
        }
        Ok(count)
    }

    #[inline]
    fn flush(&mut self) -> Result<(), Self::Error> {
        // TODO handle error
        while !self.base.lsr().read().is_transmitter_fifo_empty() {
            core::hint::spin_loop();
        }
        Ok(())
    }
}

/// Serial configuration.
#[derive(Copy, Clone, Debug, PartialEq)]
pub struct Config {
    /// Parity settings.
    pub parity: Parity,
    /// Serial stop bits.
    pub stop_bits: StopBits,
    /// Data word length.
    pub word_length: WordLength,
}

impl Default for Config {
    /// Serial configuration defaults to 8-bit word, no parity check, 1 stop bit.
    #[inline]
    fn default() -> Self {
        Config {
            parity: Parity::None,
            stop_bits: StopBits::One,
            word_length: WordLength::Eight,
        }
    }
}

/// Parity check.
#[derive(Copy, Clone, Debug, PartialEq)]
pub enum Parity {
    /// No parity check.
    None,
    /// Even parity bit.
    Even,
    /// Odd parity bit.
    Odd,
}

/// Stop bits.
#[derive(Copy, Clone, Debug, PartialEq)]
pub enum StopBits {
    /// 1 stop bit.
    One,
    /// 2 stop bits.
    Two,
}

/// Word length.
#[derive(Copy, Clone, Debug, PartialEq)]
pub enum WordLength {
    /// Five bits per word.
    Five,
    /// Six bits per word.
    Six,
    /// Seven bits per word.
    Seven,
    /// Eight bits per word.
    Eight,
}
