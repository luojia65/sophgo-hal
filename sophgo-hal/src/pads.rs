use volatile_register::RW;

pub struct RegisterBlock {
    pub fmux_sd0_clk: RW<u32>,
    pub fmux_sd0_cmd: RW<u32>,
    // TODO other fields ...
    // TODO padding
    pub io_g10_sd0_clk: RW<PadConfig>,
    pub io_g10_sd0_cmd: RW<PadConfig>,
}

impl RegisterBlock {
    #[inline]
    pub const fn fmux<const N: usize>(&self) -> &RW<u32> {
        match N {
            3 => &self.fmux_sd0_clk,
            4 => &self.fmux_sd0_cmd,
            _ => todo!(),
        }
    }
    #[inline]
    pub const fn pad_config<const N: usize>(&self) -> &RW<PadConfig> {
        match N {
            3 => &self.io_g10_sd0_clk,
            4 => &self.io_g10_sd0_cmd,
            _ => todo!(),
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
#[repr(transparent)]
pub struct PadConfig(u32);

// TODO PadConfig functions

impl PadConfig {}

// TODO tests
