//! Comparator
use crate::{
    pac,
    rcc::{Enable, Reset, APB2},
};

// Config enums
/// Comparator power mode
pub enum PowerMode {
    HighSpeed = 0x00000000,
    MediumSpeed = 0x00000004,
    LowSpeed = 0x0000000c,
}

/// Comparator input plus (Non-inverting Input)
pub enum NonInvertingInput {
    PC5 = 0x00000000,
    PB2 = 0x00000080,
    // PA1 for STM32L41xxx/42xxx/43xxx/44xxx/45xxx/46xxx
    #[cfg(any(
        feature = "stm32l431",
        feature = "stm32l451",
        feature = "stm32l412",
        feature = "stm32l422",
        feature = "stm32l432",
        feature = "stm32l442",
        feature = "stm32l452",
        feature = "stm32l462",
        feature = "stm32l433",
        feature = "stm32l443",
    ))]
    PA1 = 0x00000100,
}

// TODO Values are based on SCALEN (0x800000) and BRGEN (0x400000) check for other MCU.
/// Comparator input minus (Inverted Input)
pub enum InvertingInput {
    /// 1/4 of Vref
    OneQuarterVref = 0x00c00000,
    /// 1/2 of Vref
    OneHalfVref = 0x00c00010,
    /// 3/4 of Vref
    ThreeQuarterVref = 0x00c00020,
    /// Vref
    Vref = 0x00800030,
    /// From DAC channel 1
    DacCh1 = 0x00000040,
    /// From DAC channel 2
    DacCh2 = 0x00000050,
    /// From the first GPIO pin connected to the comparator.
    ///
    /// The GPIO pin used depends on the MCU and comparator used.
    Io1 = 0x00000060,
    /// From the second GPIO pin connected to the comparator.
    ///
    /// The GPIO pin used depends on the MCU and comparator used.
    Io2 = 0x00000070,
}

/// Comparator hysterisis
pub enum Hysterisis {
    NoHysterisis = 0x00000000,
    LowHysteresis = 0x00010000,
    MediumHysteresis = 0x00020000,
    HighHysteresis = 0x00030000,
}

/// Comparator output polarity
///
/// When [OutputPolarity::NotInverted] is used.
/// The comparator output will be high (1) when [NonInvertingInput] has higher
/// voltage than [InvertingInput]. The comparator output will be low (0) when
/// [NonInvertingInput] has lower voltage than [InvertingInput].
///
/// When [OutputPolarity::Inverted] is used.
/// The comparator output will be high (1) when [NonInvertingInput] has lower
/// voltage than [InvertingInput]. The comparator output will be low (0) when
/// [NonInvertingInput] has higher voltage than [InvertingInput].
pub enum OutputPolarity {
    /// Comparator output will not be inverted.
    NotInverted = 0x00000000,
    /// Comparator output will be inverted.
    Inverted = 0x00008000,
}

/// Comparator blanking source
pub enum BlankingSource {
    None = 0x00000000,
    Timloc5,
}

/// Comparator devices avaiable.
pub enum CompDevice {
    /// Comparator number 1 (COMP1).
    One,
    /// Comparator number 2 (COMP2).
    #[cfg(not(any(feature = "stm32l412", feature = "stm32l422")))]
    Two,
}

// Structs
/// Initial configuration data for the comparator peripheral.
pub struct CompConfig {
    /// Comparator power mode.
    pub pwrmode: PowerMode,
    /// Comparator non-inverting input.
    pub inpsel: NonInvertingInput,
    /// Comparator inverting input.
    pub inmsel: InvertingInput,
    /// Comparator hysterisis.
    pub hyst: Hysterisis,
    /// Comparator output polarity.
    pub polarity: OutputPolarity,
    /// Comparator blanking source.
    pub blanking: BlankingSource,
}

/// Macro to write bits to the register
macro_rules! set_bit {
    ($comp:ident, $value:expr) => {
        unsafe {
            let regs = &(*pac::COMP::ptr()).$comp;
            regs.modify(|r, w| {
                let current_bits = r.bits();
                let output_bits = current_bits | $value;
                w.bits(output_bits)
            })
        }
    };
}

/// Macro to clear bits in the register
macro_rules! clear_bit {
    ($comp:ident, $value:expr) => {
        unsafe {
            let regs = &(*pac::COMP::ptr()).$comp;
            let current_bits = regs.read().bits();
            let output_bits = current_bits & !$value;
            regs.write(|w| w.bits(output_bits))
        }
    };
}

/// Macro to read bits in the register
macro_rules! read_bit {
    ($comp:ident, $value:expr) => {
        unsafe {
            let regs = &(*pac::COMP::ptr()).$comp;
            regs.read().bits() & $value
        }
    };
}

/// Macro to modify the register
macro_rules! modify_bit {
    ($comp:ident, $value:expr) => {
        unsafe {
            let regs = &(*pac::COMP::ptr()).$comp;
            regs.write(|w| w.bits($value))
        }
    };
}

/// Represents an Analog Comparator peripheral.
pub struct Comp {
    device: CompDevice,
    is_locked: bool,
}

impl Comp {
    /// Initialize the comparator peripheral. This will writes the configuration
    /// according to `cfg`.
    pub fn new(device: CompDevice, cfg: CompConfig, apb: &mut APB2) -> Self {
        let result = Self {
            device,
            is_locked: false,
        };

        let config = cfg.blanking as u32
            | cfg.hyst as u32
            | cfg.inmsel as u32
            | cfg.inpsel as u32
            | cfg.polarity as u32
            | cfg.pwrmode as u32;

        <pac::COMP>::enable(apb);
        <pac::COMP>::reset(apb);

        match result.device {
            CompDevice::One => modify_bit!(comp1_csr, config),
            CompDevice::Two => modify_bit!(comp2_csr, config),
        }
        result
    }

    /// Writes bit/bits to the regiter.
    fn set_bit(&mut self, value: u32) -> Result<(), ()>  {
        if self.is_locked {
            return Err(());
        }

        match self.device {
            CompDevice::One => set_bit!(comp1_csr, value),
            CompDevice::Two => set_bit!(comp2_csr, value),
        }

        Ok(())
    }

    /// Clears bit/bits in the register.
    ///
    /// This function will return an Error when the comparator is locked.
    fn clear_bit(&mut self, value: u32) -> Result<(), ()>  {
        if self.is_locked {
            return Err(());
        }

        match self.device {
            CompDevice::One => clear_bit!(comp1_csr, value),
            CompDevice::Two => clear_bit!(comp2_csr, value),
        }

        Ok(())
    }

    /// Read bit/bits in the register.
    ///
    /// This function will return an Error when the comparator is locked.
    fn read_bit(&self, value: u32) -> u32 {
        match self.device {
            CompDevice::One => read_bit!(comp1_csr, value),
            CompDevice::Two => read_bit!(comp2_csr, value),
        }
    }

    /// Gets the output level of the comparator
    ///
    /// The output level depends on the configuration of the comparator.
    /// If the [polarity](CompConfig::polarity) is [NotInverted](OutputPolarity::NotInverted)
    /// - It will output high (1) if the non-inverting input is higher than
    /// the output of inverting input.
    /// - It will output low (0) if the non-inverting input is lower than
    /// the output of the inverting input.
    ///
    /// The oposite will be out inverted if [polarity](CompConfig::polarity) is
    /// [Inverted](OutputPolarity::NotInverted).
    pub fn get_output_level(&self) -> u32 {
        self.read_bit(0b1 << 30) >> 30
    }

    /// Starts the comparator.
    ///
    /// This function will return an Error when the comparator is locked.
    pub fn start(&mut self) -> Result<(), ()> {
        self.set_bit(0b1)
    }

    /// Stops the comparator.
    ///
    /// This function will return an Error when the comparator is locked.
    pub fn stop(&mut self) -> Result<(), ()> {
        self.clear_bit(0b1)
    }

    /// Locks the comparator.
    ///
    /// This locks the comparator registers making it only read-only.
    ///
    /// *Note:* The lock also applies to the lock bit itself. Therefore,
    /// the comparator register/configuration *cannot* be changed until
    /// a hardware reset.
    ///
    /// This function will return an Error when the comparator is locked.
    pub fn lock(&mut self) -> Result<(), ()> {
        self.is_locked = true;
        self.set_bit(0x80000000)
    }
}
