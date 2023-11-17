use crate::kernel::syscall::*;
use crate::vm::instructions;
use crate::vm::register::*;

use rand::prelude::*;

pub const BYTECODE_SIZE: usize = 65536;
pub const MEMORY_SIZE: usize = 65536;

pub enum CpuFlag {
    Zero,
    Carry,
    Overflow,
    Sign,
    Sleep,
}

#[derive(Clone)]
pub struct Emulator {
    memory: Vec<u8>,
    registers: Registers,
    bytecode: Vec<u8>,
}

impl Emulator {
    pub fn new(memory: Vec<u8>, bytecode: Vec<u8>) -> Self {
        Emulator {
            memory,
            registers: Registers::new(),
            bytecode,
        }
    }

    pub fn run_until_interrupt(&mut self, cycle_count: &mut usize) -> Option<&'static dyn Syscall> {
        loop {
            let opcode: u8 = self.peek_from_pc();
            let instruction = instructions::OPCODE_TABLE.get_instruction(opcode);
            let latency = instruction.get_latency();
            if *cycle_count < latency {
                return None;
            }
            *cycle_count -= latency;
            self.increment_pc(1);

            if self.get_cpu_flag(CpuFlag::Sleep)
                && !instructions::OpcodeTable::is_valid_sleep_opcode(opcode)
            {
                self.nasal_demons();
                self.set_cpu_flag(CpuFlag::Sleep, false);
            }

            if opcode == instructions::OpcodeTable::get_syscall_opcode() {
                return Some(SYSCALL_TABLE.get_syscall(self.get_syscall_number()));
            }
            instruction.execute(self);
        }
    }

    fn get_reg_internal(&self, register: RegisterName) -> u16 {
        self.registers.get(register).get_value_internal()
    }

    pub fn get_reg_mut(&mut self, register: RegisterName) -> u16 {
        self.registers.get_mut(register).get_value_mut()
    }

    pub fn set_reg(&mut self, register: RegisterName, value: u16) {
        self.registers.get_mut(register).set_value(value)
    }

    pub fn increment_pc(&mut self, count: u16) {
        let pc_value = self.get_reg_internal(RegisterName::PC);
        self.set_reg(RegisterName::PC, pc_value.wrapping_add(count));
    }

    fn peek_bytes_from_pc_fixed<const N: usize>(&self) -> [u8; N] {
        let mut result = [0; N];
        let mut pc_value = self.get_reg_internal(RegisterName::PC) as usize;
        for byte in result.iter_mut() {
            *byte = self.bytecode[pc_value];
            pc_value = pc_value.wrapping_add(1) % BYTECODE_SIZE;
        }
        result
    }

    pub fn peek_from_pc<T: FromBytes<Bytes = [u8; N]>, const N: usize>(&self) -> T {
        T::from_bytes(&self.peek_bytes_from_pc_fixed::<N>())
    }

    fn read_bytes_from_pc_fixed<const N: usize>(&mut self) -> [u8; N] {
        let result = self.peek_bytes_from_pc_fixed();
        self.increment_pc(N as u16);
        result
    }

    pub fn read_from_pc<T: FromBytes<Bytes = [u8; N]>, const N: usize>(&mut self) -> T {
        T::from_bytes(&self.read_bytes_from_pc_fixed::<N>())
    }

    pub fn peek_bytes_from_mem(&self, addr: u16, count: usize) -> Vec<u8> {
        let mut result = vec![0; count];
        let mut addr = addr as usize;
        for byte in result.iter_mut() {
            *byte = self.memory[addr];
            addr = addr.wrapping_add(1) % MEMORY_SIZE;
        }
        result
    }

    fn peek_bytes_from_mem_fixed<const N: usize>(&self, addr: u16) -> [u8; N] {
        let mut result = [0; N];
        let mut addr = addr as usize;
        for byte in result.iter_mut() {
            *byte = self.memory[addr];
            addr = addr.wrapping_add(1) % MEMORY_SIZE;
        }
        result
    }

    pub fn peek_from_mem<T: FromBytes<Bytes = [u8; N]>, const N: usize>(&self, addr: u16) -> T {
        T::from_bytes(&self.peek_bytes_from_mem_fixed::<N>(addr))
    }

    pub fn write_bytes_to_mem(&mut self, addr: u16, bytes: &[u8]) {
        let mut addr = addr as usize;
        for byte in bytes {
            self.memory[addr] = *byte;
            addr = addr.wrapping_add(1) % MEMORY_SIZE;
        }
    }

    pub fn write_to_mem<T: ToBytes<Bytes = [u8; N]>, const N: usize>(
        &mut self,
        addr: u16,
        data: T,
    ) {
        self.write_bytes_to_mem(addr, &data.to_bytes())
    }

    pub fn write_bytes_to_code(&mut self, addr: u16, bytes: &[u8]) {
        let mut addr = addr as usize;
        for byte in bytes {
            self.bytecode[addr] = *byte;
            addr = addr.wrapping_add(1) % BYTECODE_SIZE;
        }
    }

    pub fn read_registers_operand(&mut self) -> (RegisterName, RegisterName) {
        let indices: u8 = self.read_from_pc();
        let index_lo = indices & 0xf;
        let index_hi = indices >> 4;
        (index_lo.into(), index_hi.into())
    }

    pub fn read_address_operand(&mut self) -> u16 {
        let mode_base: u8 = self.read_from_pc();
        let mode = mode_base >> 6;
        let base = self.get_reg_mut((mode_base & 0xf).into());
        match mode {
            0b00 => base,
            0b01 => {
                let displacement = self.read_from_pc();
                base.wrapping_add(displacement)
            }
            0b10 => {
                let scale_index: u8 = self.read_from_pc();
                let index = self.get_reg_mut((scale_index & 0xf).into());
                let scale = (scale_index >> 4).next_power_of_two() as u16;
                base.wrapping_add(index.wrapping_mul(scale))
            }
            0b11 => {
                let scale_index: u8 = self.read_from_pc();
                let index = self.get_reg_mut((scale_index & 0xf).into());
                let scale = (scale_index >> 4).next_power_of_two() as u16;
                let displacement = self.read_from_pc();
                base.wrapping_add(index.wrapping_mul(scale))
                    .wrapping_add(displacement)
            }
            _ => unreachable!(),
        }
    }

    pub fn get_cpu_flag(&self, flag: CpuFlag) -> bool {
        let flags_reg = self.get_reg_internal(RegisterName::FL);
        let flag_idx = flag as u32;
        flags_reg & (1 << flag_idx) == 1 << flag_idx
    }

    pub fn set_cpu_flag(&mut self, flag: CpuFlag, value: bool) {
        let flags_reg = self.get_reg_internal(RegisterName::FL);
        let flag_idx = flag as u32;
        let new_flags = if value {
            flags_reg | (1 << flag_idx)
        } else {
            flags_reg & !(1 << flag_idx)
        };
        self.set_reg(RegisterName::FL, new_flags);
    }

    pub fn set_logical_flags(&mut self, value: u16) {
        self.set_cpu_flag(CpuFlag::Zero, value == 0);
        self.set_cpu_flag(CpuFlag::Sign, (value as i16) < 0);
    }

    pub fn set_arithmetic_flags(&mut self, value: u16, carry: bool, overflow: bool) {
        self.set_cpu_flag(CpuFlag::Carry, carry);
        self.set_cpu_flag(CpuFlag::Overflow, overflow);
        self.set_logical_flags(value);
    }

    pub fn nasal_demons(&mut self) {
        let mut rng = rand::thread_rng();
        match rng.gen_range(1..=100) {
            1..=40 => {
                let addr = rng.gen();
                self.write_bytes_to_mem(addr, &[rng.gen()]);
            }
            41..=60 => {
                let reg = RegisterName::from(rng.gen_range(0..=0xf));
                self.set_reg(reg, rng.gen());
            }
            61..=85 => {
                let reg1 = RegisterName::from(rng.gen_range(0..=0xf));
                let reg2 = RegisterName::from(rng.gen_range(0..=0xf));
                let value1 = self.get_reg_mut(reg1);
                let value2 = self.get_reg_mut(reg2);
                self.set_reg(reg1, value2);
                self.set_reg(reg2, value1);
            }
            86..=100 => {
                let addr = rng.gen();
                self.write_bytes_to_code(addr, &[rng.gen()]);
            }
            _ => unreachable!(),
        }
    }

    fn get_syscall_number(&self) -> u8 {
        self.get_reg_internal(RegisterName::AX) as u8
    }

    pub fn get_syscall_args(&self) -> SyscallArgs {
        let r0 = self.get_reg_internal(RegisterName::R0);
        let r1 = self.get_reg_internal(RegisterName::R1);
        let r2 = self.get_reg_internal(RegisterName::R2);
        let r3 = self.get_reg_internal(RegisterName::R3);
        let r4 = self.get_reg_internal(RegisterName::R4);
        let r5 = self.get_reg_internal(RegisterName::R5);
        (r0, r1, r2, r3, r4, r5)
    }

    pub fn set_syscall_return_value(&mut self, value: u16) {
        self.set_reg(RegisterName::AX, value);
    }

    pub fn increment_ts(&mut self) {
        let ts = self.get_reg_internal(RegisterName::TS);
        self.set_reg(RegisterName::TS, ts.wrapping_add(1));
    }
}

pub trait FromBytes {
    type Bytes;
    fn from_bytes(bytes: &Self::Bytes) -> Self;
}

pub trait ToBytes {
    type Bytes;
    fn to_bytes(&self) -> Self::Bytes;
}

macro_rules! impl_from_bytes {
    ($($type:ty),* $(,)?) => {
        $(
            impl FromBytes for $type {
                type Bytes = [u8; std::mem::size_of::<$type>()];
                fn from_bytes(bytes: &Self::Bytes) -> Self {
                    <$type>::from_le_bytes(*bytes)
                }
            }
        )*
    };
}

macro_rules! impl_to_bytes {
    ($($type:ty),+) => {
        $(
            impl ToBytes for $type {
                type Bytes = [u8; std::mem::size_of::<$type>()];
                fn to_bytes(&self) -> Self::Bytes {
                    self.to_le_bytes()
                }
            }
        )+
    };
}

impl_from_bytes! {
    i8, i16, i32, i64, i128, u8, u16, u32, u64, u128
}

impl_to_bytes! {
    i8, i16, i32, i64, i128, u8, u16, u32, u64, u128
}
