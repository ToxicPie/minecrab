use rand::{rngs::StdRng, Rng, SeedableRng};

#[derive(Copy, Clone)]
pub enum RegisterName {
    PC,
    FL,
    CT,
    R0,
    R1,
    R2,
    R3,
    R4,
    R5,
    SP,
    TF,
    ZR,
    RR,
    TS,
    RE,
    AX,
}

impl From<u8> for RegisterName {
    fn from(value: u8) -> Self {
        match value & 0xf {
            0x0 => RegisterName::PC,
            0x1 => RegisterName::FL,
            0x2 => RegisterName::CT,
            0x3 => RegisterName::R0,
            0x4 => RegisterName::R1,
            0x5 => RegisterName::R2,
            0x6 => RegisterName::R3,
            0x7 => RegisterName::R4,
            0x8 => RegisterName::R5,
            0x9 => RegisterName::SP,
            0xa => RegisterName::TF,
            0xb => RegisterName::ZR,
            0xc => RegisterName::RR,
            0xd => RegisterName::TS,
            0xe => RegisterName::RE,
            0xf => RegisterName::AX,
            _ => unreachable!(),
        }
    }
}

pub trait Register {
    fn get_value_internal(&self) -> u16;
    fn get_value_mut(&mut self) -> u16;
    fn set_value(&mut self, value: u16);
}

pub struct Registers {
    registers: [Box<dyn Register>; 16],
}

impl Registers {
    pub fn new() -> Self {
        Self {
            registers: [
                Box::<GeneralPurposeRegister>::default(), // PC
                Box::<GeneralPurposeRegister>::default(), // FL
                Box::<CounterRegister>::default(),        // CT
                Box::<GeneralPurposeRegister>::default(), // R0
                Box::<GeneralPurposeRegister>::default(), // R1
                Box::<GeneralPurposeRegister>::default(), // R2
                Box::<GeneralPurposeRegister>::default(), // R3
                Box::<GeneralPurposeRegister>::default(), // R4
                Box::<GeneralPurposeRegister>::default(), // R5
                Box::<GeneralPurposeRegister>::default(), // SP
                Box::<ConstRegister<0x1337>>::default(),  // TF
                Box::<ConstRegister<0>>::default(),       // ZR
                Box::<RandomValueRegister>::default(),    // RR
                Box::<GeneralPurposeRegister>::default(), // TS
                Box::<BitRevRegister>::default(),         // RE
                Box::<GeneralPurposeRegister>::default(), // AX
            ],
        }
    }

    pub fn get(&self, register: RegisterName) -> &dyn Register {
        self.registers[register as usize].as_ref()
    }

    pub fn get_mut(&mut self, register: RegisterName) -> &mut dyn Register {
        self.registers[register as usize].as_mut()
    }
}

impl Clone for Registers {
    fn clone(&self) -> Self {
        let mut result = Self::new();
        for (dst, src) in result.registers.iter_mut().zip(self.registers.iter()) {
            dst.set_value(src.get_value_internal());
        }
        result
    }
}

#[derive(Default)]
pub struct GeneralPurposeRegister {
    value: u16,
}

#[derive(Default)]
pub struct ConstRegister<const N: u16> {}

pub struct RandomValueRegister {
    rng: StdRng,
}

#[derive(Default)]
pub struct CounterRegister {
    value: u16,
}

#[derive(Default)]
pub struct BitRevRegister {
    value: u16,
}

impl Register for GeneralPurposeRegister {
    fn get_value_internal(&self) -> u16 {
        self.value
    }
    fn get_value_mut(&mut self) -> u16 {
        self.value
    }
    fn set_value(&mut self, value: u16) {
        self.value = value;
    }
}

impl<const N: u16> Register for ConstRegister<N> {
    fn get_value_internal(&self) -> u16 {
        N
    }
    fn get_value_mut(&mut self) -> u16 {
        N
    }
    fn set_value(&mut self, _value: u16) {}
}

impl Default for RandomValueRegister {
    fn default() -> Self {
        Self {
            rng: StdRng::from_entropy(),
        }
    }
}

impl Register for RandomValueRegister {
    fn get_value_internal(&self) -> u16 {
        0
    }
    fn get_value_mut(&mut self) -> u16 {
        self.rng.gen()
    }
    fn set_value(&mut self, _value: u16) {}
}

impl Register for CounterRegister {
    fn get_value_internal(&self) -> u16 {
        self.value
    }
    fn get_value_mut(&mut self) -> u16 {
        let old = self.value;
        self.value = self.value.wrapping_add(1);
        old
    }
    fn set_value(&mut self, value: u16) {
        self.value = value;
    }
}

impl Register for BitRevRegister {
    fn get_value_internal(&self) -> u16 {
        self.value
    }
    fn get_value_mut(&mut self) -> u16 {
        self.value.reverse_bits()
    }
    fn set_value(&mut self, value: u16) {
        self.value = value;
    }
}
