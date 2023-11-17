use crate::vm::emulator::CpuFlag;
use crate::vm::emulator::Emulator;
use crate::vm::register::RegisterName;

pub trait Instruction {
    fn get_opcode(&self) -> u8;
    fn get_latency(&self) -> usize;
    fn execute(&self, emulator: &mut Emulator);
}

pub struct OpcodeTable {
    _used: [bool; Self::TABLE_SIZE],
    table: [&'static dyn Instruction; Self::TABLE_SIZE],
}

impl OpcodeTable {
    const TABLE_SIZE: usize = 256;
    pub const fn new() -> Self {
        Self {
            _used: [false; Self::TABLE_SIZE],
            table: [&Reserved {}; Self::TABLE_SIZE],
        }
    }
    pub fn get_syscall_opcode() -> u8 {
        Syscall {}.get_opcode()
    }
    pub fn is_valid_sleep_opcode(opcode: u8) -> bool {
        opcode == Op {}.get_opcode() || opcode == P {}.get_opcode()
    }
    pub fn get_instruction(&self, opcode: u8) -> &'static dyn Instruction {
        self.table[opcode as usize]
    }
}

macro_rules! instruction_category {
    ($category_fn:ident() => [
        $($mnemonic:ident<$opcode:literal, $latency:literal> ($arg:ident) $exec:tt),* $(,)?
    ]) => {
        $(
            struct $mnemonic {}
            impl Instruction for $mnemonic {
                fn get_opcode(&self) -> u8 {
                    $opcode
                }
                fn get_latency(&self) -> usize {
                    $latency
                }
                fn execute(&self, $arg: &mut Emulator) {
                    $exec
                }
            }
        )*
        impl OpcodeTable {
            pub const fn $category_fn(mut self) -> Self {
                $(
                    if self._used[$opcode as usize] {
                        panic!(concat!("duplicate opcode ", stringify!($opcode)));
                    }
                    self._used[$opcode as usize] = true;
                    self.table[$opcode as usize] = &$mnemonic {};
                )*
                self
            }
        }
    };
}

pub const OPCODE_TABLE: OpcodeTable = OpcodeTable::new()
    .make_data_instructions()
    .make_arithmetic_instructions()
    .make_logical_instructions()
    .make_control_flow_instrucions()
    .make_misc_instructions();

instruction_category! {
    make_data_instructions() => [
        MovReg8<0x23, 3>(emulator) {
            let (dst, src) = emulator.read_registers_operand();
            let value = emulator.get_reg_mut(src) as u8;
            emulator.set_reg(dst, value as u16);
        },
        MovReg16<0x22, 3>(emulator) {
            let (dst, src) = emulator.read_registers_operand();
            let value = emulator.get_reg_mut(src);
            emulator.set_reg(dst, value);
        },
        MovImm8<0x21, 3>(emulator) {
            let (dst, _) = emulator.read_registers_operand();
            let value: u8 = emulator.read_from_pc();
            emulator.set_reg(dst, value as u16);
        },
        MovImm16<0x20, 4>(emulator) {
            let (dst, _) = emulator.read_registers_operand();
            let value = emulator.read_from_pc();
            emulator.set_reg(dst, value);
        },

        Load8<0x25, 24>(emulator) {
            let (dst, _) = emulator.read_registers_operand();
            let addr = emulator.read_address_operand();
            let value: u8 = emulator.peek_from_mem(addr);
            emulator.set_reg(dst, value as u16);
        },
        Load16<0x24, 28>(emulator) {
            let (dst, _) = emulator.read_registers_operand();
            let addr = emulator.read_address_operand();
            let value = emulator.peek_from_mem(addr);
            emulator.set_reg(dst, value);
        },

        StoreReg8<0x29, 22>(emulator) {
            let addr = emulator.read_address_operand();
            let (src, _) = emulator.read_registers_operand();
            let value = emulator.get_reg_mut(src) as u8;
            emulator.write_to_mem(addr, value);
        },
        StoreReg16<0x28, 26>(emulator) {
            let addr = emulator.read_address_operand();
            let (src, _) = emulator.read_registers_operand();
            let value = emulator.get_reg_mut(src);
            emulator.write_to_mem(addr, value);
        },
        StoreImm8<0x27, 24>(emulator) {
            let addr = emulator.read_address_operand();
            let value: u8 = emulator.read_from_pc();
            emulator.write_to_mem(addr, value);
        },
        StoreImm16<0x26, 28>(emulator) {
            let addr = emulator.read_address_operand();
            let value: u16 = emulator.read_from_pc();
            emulator.write_to_mem(addr, value);
        },

        Lea<0x8d, 5>(emulator) {
            let (dst, _) = emulator.read_registers_operand();
            let addr = emulator.read_address_operand();
            emulator.set_reg(dst, addr);
        },

        Xchg<0x92, 3>(emulator) {
            let (reg1, reg2) = emulator.read_registers_operand();
            let val1 = emulator.get_reg_mut(reg1);
            let val2 = emulator.get_reg_mut(reg2);
            emulator.set_reg(reg1, val2);
            emulator.set_reg(reg2, val1);
        },

        Sex<0x62, 3>(emulator) {
            let (reg, _) = emulator.read_registers_operand();
            let val = emulator.get_reg_mut(reg) as i8;
            emulator.set_reg(reg, val as u16);
        },

        CmovaReg16<0xd6, 8>(emulator) {
            let cf = emulator.get_cpu_flag(CpuFlag::Carry);
            let zf = emulator.get_cpu_flag(CpuFlag::Zero);
            if !cf && !zf {
                MovReg16 {}.execute(emulator);
            } else {
                emulator.increment_pc(1);
            }
        },
        CmovaImm16<0xe6, 9>(emulator) {
            let cf = emulator.get_cpu_flag(CpuFlag::Carry);
            let zf = emulator.get_cpu_flag(CpuFlag::Zero);
            if !cf && !zf {
                MovImm16 {}.execute(emulator);
            } else {
                emulator.increment_pc(3);
            }
        },
        CmovaeReg16<0xd7, 8>(emulator) {
            let cf = emulator.get_cpu_flag(CpuFlag::Carry);
            if !cf {
                MovReg16 {}.execute(emulator);
            } else {
                emulator.increment_pc(1);
            }
        },
        CmovaeImm16<0xe7, 9>(emulator) {
            let cf = emulator.get_cpu_flag(CpuFlag::Carry);
            if !cf {
                MovImm16 {}.execute(emulator);
            } else {
                emulator.increment_pc(3);
            }
        },
        CmovbReg16<0xd8, 8>(emulator) {
            let cf = emulator.get_cpu_flag(CpuFlag::Carry);
            if cf {
                MovReg16 {}.execute(emulator);
            } else {
                emulator.increment_pc(1);
            }
        },
        CmovbImm16<0xe8, 9>(emulator) {
            let cf = emulator.get_cpu_flag(CpuFlag::Carry);
            if cf {
                MovImm16 {}.execute(emulator);
            } else {
                emulator.increment_pc(3);
            }
        },
        CmovbeReg16<0xd9, 8>(emulator) {
            let cf = emulator.get_cpu_flag(CpuFlag::Carry);
            let zf = emulator.get_cpu_flag(CpuFlag::Zero);
            if cf || zf {
                MovReg16 {}.execute(emulator);
            } else {
                emulator.increment_pc(1);
            }
        },
        CmovbeImm16<0xe9, 9>(emulator) {
            let cf = emulator.get_cpu_flag(CpuFlag::Carry);
            let zf = emulator.get_cpu_flag(CpuFlag::Zero);
            if cf || zf {
                MovImm16 {}.execute(emulator);
            } else {
                emulator.increment_pc(3);
            }
        },
        CmoveReg16<0xda, 8>(emulator) {
            let zf = emulator.get_cpu_flag(CpuFlag::Zero);
            if zf {
                MovReg16 {}.execute(emulator);
            } else {
                emulator.increment_pc(1);
            }
        },
        CmoveImm16<0xea, 9>(emulator) {
            let zf = emulator.get_cpu_flag(CpuFlag::Zero);
            if zf {
                MovImm16 {}.execute(emulator);
            } else {
                emulator.increment_pc(3);
            }
        },
        CmovneReg16<0xdb, 8>(emulator) {
            let zf = emulator.get_cpu_flag(CpuFlag::Zero);
            if !zf {
                MovReg16 {}.execute(emulator);
            } else {
                emulator.increment_pc(1);
            }
        },
        CmovneImm16<0xeb, 9>(emulator) {
            let zf = emulator.get_cpu_flag(CpuFlag::Zero);
            if !zf {
                MovImm16 {}.execute(emulator);
            } else {
                emulator.increment_pc(3);
            }
        },
        CmovgReg16<0xdc, 8>(emulator) {
            let zf = emulator.get_cpu_flag(CpuFlag::Zero);
            let sf = emulator.get_cpu_flag(CpuFlag::Sign);
            let of = emulator.get_cpu_flag(CpuFlag::Overflow);
            if !zf && sf == of {
                MovReg16 {}.execute(emulator);
            } else {
                emulator.increment_pc(1);
            }
        },
        CmovgImm16<0xec, 9>(emulator) {
            let zf = emulator.get_cpu_flag(CpuFlag::Zero);
            let sf = emulator.get_cpu_flag(CpuFlag::Sign);
            let of = emulator.get_cpu_flag(CpuFlag::Overflow);
            if !zf && sf == of {
                MovImm16 {}.execute(emulator);
            } else {
                emulator.increment_pc(3);
            }
        },
        CmovgeReg16<0xdd, 8>(emulator) {
            let sf = emulator.get_cpu_flag(CpuFlag::Sign);
            let of = emulator.get_cpu_flag(CpuFlag::Overflow);
            if sf == of {
                MovReg16 {}.execute(emulator);
            } else {
                emulator.increment_pc(1);
            }
        },
        CmovgeImm16<0xed, 9>(emulator) {
            let sf = emulator.get_cpu_flag(CpuFlag::Sign);
            let of = emulator.get_cpu_flag(CpuFlag::Overflow);
            if sf == of {
                MovImm16 {}.execute(emulator);
            } else {
                emulator.increment_pc(3);
            }
        },
        CmovlReg16<0xde, 8>(emulator) {
            let sf = emulator.get_cpu_flag(CpuFlag::Sign);
            let of = emulator.get_cpu_flag(CpuFlag::Overflow);
            if sf != of {
                MovReg16 {}.execute(emulator);
            } else {
                emulator.increment_pc(1);
            }
        },
        CmovlImm16<0xee, 9>(emulator) {
            let sf = emulator.get_cpu_flag(CpuFlag::Sign);
            let of = emulator.get_cpu_flag(CpuFlag::Overflow);
            if sf != of {
                MovImm16 {}.execute(emulator);
            } else {
                emulator.increment_pc(3);
            }
        },
        CmovleReg16<0xdf, 8>(emulator) {
            let zf = emulator.get_cpu_flag(CpuFlag::Zero);
            let sf = emulator.get_cpu_flag(CpuFlag::Sign);
            let of = emulator.get_cpu_flag(CpuFlag::Overflow);
            if zf || sf != of {
                MovReg16 {}.execute(emulator);
            } else {
                emulator.increment_pc(1);
            }
        },
        CmovleImm16<0xef, 9>(emulator) {
            let zf = emulator.get_cpu_flag(CpuFlag::Zero);
            let sf = emulator.get_cpu_flag(CpuFlag::Sign);
            let of = emulator.get_cpu_flag(CpuFlag::Overflow);
            if zf || sf != of {
                MovImm16 {}.execute(emulator);
            } else {
                emulator.increment_pc(3);
            }
        },

        PushReg8<0x50, 24>(emulator) {
            let (src, _) = emulator.read_registers_operand();
            let value = emulator.get_reg_mut(src) as u8;
            let sp_value = emulator.get_reg_mut(RegisterName::SP);
            emulator.write_to_mem(sp_value, value);
            emulator.set_reg(RegisterName::SP, sp_value.wrapping_add(1));
        },
        PushReg16<0x51, 28>(emulator) {
            let (src, _) = emulator.read_registers_operand();
            let value = emulator.get_reg_mut(src);
            let sp_value = emulator.get_reg_mut(RegisterName::SP);
            emulator.write_to_mem(sp_value, value);
            emulator.set_reg(RegisterName::SP, sp_value.wrapping_add(2));
        },
        PushImm8<0x52, 26>(emulator) {
            let value: u8 = emulator.read_from_pc();
            let sp_value = emulator.get_reg_mut(RegisterName::SP);
            emulator.write_to_mem(sp_value, value);
            emulator.set_reg(RegisterName::SP, sp_value.wrapping_add(1));
        },
        PushImm16<0x53, 30>(emulator) {
            let value: u16 = emulator.read_from_pc();
            let sp_value = emulator.get_reg_mut(RegisterName::SP);
            emulator.write_to_mem(sp_value, value);
            emulator.set_reg(RegisterName::SP, sp_value.wrapping_add(2));
        },

        Pop8<0x60, 26>(emulator) {
            let (dst, _) = emulator.read_registers_operand();
            let sp_value = emulator.get_reg_mut(RegisterName::SP);
            emulator.set_reg(RegisterName::SP, sp_value.wrapping_sub(1));
            let value: u8 = emulator.peek_from_mem(sp_value.wrapping_sub(1));
            emulator.set_reg(dst, value as u16);
        },
        Pop16<0x61, 30>(emulator) {
            let (dst, _) = emulator.read_registers_operand();
            let sp_value = emulator.get_reg_mut(RegisterName::SP);
            emulator.set_reg(RegisterName::SP, sp_value.wrapping_sub(2));
            let value = emulator.peek_from_mem(sp_value.wrapping_sub(2));
            emulator.set_reg(dst, value);
        },

        Memcpy<0x71, 512>(emulator) {
            let (nreg, _) = emulator.read_registers_operand();
            let n = (emulator.get_reg_mut(nreg) & 0xff) + 1;
            let dst = emulator.read_address_operand();
            let src = emulator.read_address_operand();
            if dst.wrapping_sub(src) < n || src.wrapping_sub(dst) < n {
                emulator.nasal_demons();
                return;
            }
            let bytes = emulator.peek_bytes_from_mem(src, n as usize);
            emulator.write_bytes_to_mem(dst, &bytes);
        },

        MemsetReg<0x81, 384>(emulator) {
            let (nreg, _) = emulator.read_registers_operand();
            let n = (emulator.get_reg_mut(nreg) & 0xff) + 1;
            let dst = emulator.read_address_operand();
            let (breg, _) = emulator.read_registers_operand();
            let b = emulator.get_reg_mut(breg) as u8;
            let bytes = vec![b; n as usize];
            emulator.write_bytes_to_mem(dst, &bytes);
        },
        MemsetImm8<0x82, 384>(emulator) {
            let (nreg, _) = emulator.read_registers_operand();
            let n = (emulator.get_reg_mut(nreg) & 0xff) + 1;
            let dst = emulator.read_address_operand();
            let b: u8 = emulator.read_from_pc();
            let bytes = vec![b; n as usize];
            emulator.write_bytes_to_mem(dst, &bytes);
        },
    ]
}

instruction_category! {
    make_arithmetic_instructions() => [
        AddReg16<0xc4, 4>(emulator) {
            let (dst, src) = emulator.read_registers_operand();
            let val1 = emulator.get_reg_mut(dst);
            let val2 = emulator.get_reg_mut(src);
            let (sum, carry) = val1.overflowing_add(val2);
            let (_, overflow) = (val1 as i16).overflowing_add(val2 as i16);
            emulator.set_arithmetic_flags(sum, carry, overflow);
            emulator.set_reg(dst, sum);
        },
        AddImm8<0xc5, 4>(emulator) {
            let (dst, _) = emulator.read_registers_operand();
            let val1 = emulator.get_reg_mut(dst);
            let val2 = emulator.read_from_pc::<i8, 1>() as u16;
            let (sum, carry) = val1.overflowing_add(val2);
            let (_, overflow) = (val1 as i16).overflowing_add(val2 as i16);
            emulator.set_arithmetic_flags(sum, carry, overflow);
            emulator.set_reg(dst, sum);
        },
        AddImm16<0xc6, 4>(emulator) {
            let (dst, _) = emulator.read_registers_operand();
            let val1 = emulator.get_reg_mut(dst);
            let val2 = emulator.read_from_pc();
            let (sum, carry) = val1.overflowing_add(val2);
            let (_, overflow) = (val1 as i16).overflowing_add(val2 as i16);
            emulator.set_arithmetic_flags(sum, carry, overflow);
            emulator.set_reg(dst, sum);
        },

        SubReg16<0xb4, 4>(emulator) {
            let (dst, src) = emulator.read_registers_operand();
            let val1 = emulator.get_reg_mut(dst);
            let val2 = emulator.get_reg_mut(src);
            let (sum, carry) = val1.overflowing_sub(val2);
            let (_, overflow) = (val1 as i16).overflowing_sub(val2 as i16);
            emulator.set_arithmetic_flags(sum, carry, overflow);
            emulator.set_reg(dst, sum);
        },
        SubImm8<0xb5, 4>(emulator) {
            let (dst, _) = emulator.read_registers_operand();
            let val1 = emulator.get_reg_mut(dst);
            let val2 = emulator.read_from_pc::<i8, 1>() as u16;
            let (sum, carry) = val1.overflowing_sub(val2);
            let (_, overflow) = (val1 as i16).overflowing_sub(val2 as i16);
            emulator.set_arithmetic_flags(sum, carry, overflow);
            emulator.set_reg(dst, sum);
        },
        SubImm16<0xb6, 4>(emulator) {
            let (dst, _) = emulator.read_registers_operand();
            let val1 = emulator.get_reg_mut(dst);
            let val2 = emulator.read_from_pc();
            let (sum, carry) = val1.overflowing_sub(val2);
            let (_, overflow) = (val1 as i16).overflowing_sub(val2 as i16);
            emulator.set_arithmetic_flags(sum, carry, overflow);
            emulator.set_reg(dst, sum);
        },

        MulReg16<0xa4, 18>(emulator) {
            let (dst1, dst2) = emulator.read_registers_operand();
            let (src, _) = emulator.read_registers_operand();
            let val1 = emulator.get_reg_mut(dst1);
            let val2 = emulator.get_reg_mut(src);
            let (prod_lo, prod_hi) = val1.widening_mul(val2);
            emulator.set_arithmetic_flags(prod_lo, prod_hi != 0, prod_hi != 0);
            emulator.set_reg(dst1, prod_lo);
            emulator.set_reg(dst2, prod_hi);
        },
        MulImm8<0xa5, 16>(emulator) {
            let (dst1, dst2) = emulator.read_registers_operand();
            let val1 = emulator.get_reg_mut(dst1);
            let val2 = emulator.read_from_pc::<u8, 1>() as u16;
            let (prod_lo, prod_hi) = val1.widening_mul(val2);
            emulator.set_arithmetic_flags(prod_lo, prod_hi != 0, prod_hi != 0);
            emulator.set_reg(dst1, prod_lo);
            emulator.set_reg(dst2, prod_hi);
        },
        MulImm16<0xa6, 18>(emulator) {
            let (dst1, dst2) = emulator.read_registers_operand();
            let val1 = emulator.get_reg_mut(dst1);
            let val2 = emulator.read_from_pc();
            let (prod_lo, prod_hi) = val1.widening_mul(val2);
            emulator.set_arithmetic_flags(prod_lo, prod_hi != 0, prod_hi != 0);
            emulator.set_reg(dst1, prod_lo);
            emulator.set_reg(dst2, prod_hi);
        },


        MulloReg16<0x94, 12>(emulator) {
            let (dst, src) = emulator.read_registers_operand();
            let val1 = emulator.get_reg_mut(dst);
            let val2 = emulator.get_reg_mut(src);
            let (prod, carry) = val1.overflowing_mul(val2);
            let (_, overflow) = (val1 as i16).overflowing_mul(val2 as i16);
            emulator.set_arithmetic_flags(prod, carry, overflow);
            emulator.set_reg(dst, prod);
        },
        MulloImm8<0x95, 10>(emulator) {
            let (dst, _) = emulator.read_registers_operand();
            let val1 = emulator.get_reg_mut(dst);
            let val2 = emulator.read_from_pc::<u8, 1>() as u16;
            let (prod, carry) = val1.overflowing_mul(val2);
            let (_, overflow) = (val1 as i16).overflowing_mul(val2 as i16);
            emulator.set_arithmetic_flags(prod, carry, overflow);
            emulator.set_reg(dst, prod);
        },
        MulloImm16<0x96, 12>(emulator) {
            let (dst, _) = emulator.read_registers_operand();
            let val1 = emulator.get_reg_mut(dst);
            let val2 = emulator.read_from_pc();
            let (prod, carry) = val1.overflowing_mul(val2);
            let (_, overflow) = (val1 as i16).overflowing_mul(val2 as i16);
            emulator.set_arithmetic_flags(prod, carry, overflow);
            emulator.set_reg(dst, prod);
        },

        ImulReg16<0x84, 18>(emulator) {
            let (dst1, dst2) = emulator.read_registers_operand();
            let (src, _) = emulator.read_registers_operand();
            let val1: i16 = emulator.get_reg_mut(dst1) as i16;
            let val2: i16 = emulator.get_reg_mut(src) as i16;
            let (prod_lo, prod_hi, overflow) = {
                let prod = (val1 as i32) * (val2 as i32);
                let overflow = i16::try_from(prod).is_err();
                ((prod >> 16) as u16, prod as u16, overflow)
            };
            emulator.set_arithmetic_flags(prod_lo, overflow, overflow);
            emulator.set_reg(dst1, prod_lo);
            emulator.set_reg(dst2, prod_hi);
        },
        ImulImm8<0x85, 16>(emulator) {
            let (dst1, dst2) = emulator.read_registers_operand();
            let val1: i16 = emulator.get_reg_mut(dst1) as i16;
            let val2: i16 = emulator.read_from_pc::<i8, 1>() as i16;
            let (prod_lo, prod_hi, overflow) = {
                let prod = (val1 as i32) * (val2 as i32);
                let overflow = i16::try_from(prod).is_err();
                ((prod >> 16) as u16, prod as u16, overflow)
            };
            emulator.set_arithmetic_flags(prod_lo, overflow, overflow);
            emulator.set_reg(dst1, prod_lo);
            emulator.set_reg(dst2, prod_hi);
        },
        ImulImm16<0x86, 18>(emulator) {
            let (dst1, dst2) = emulator.read_registers_operand();
            let val1: i16 = emulator.get_reg_mut(dst1) as i16;
            let val2: i16 = emulator.read_from_pc();
            let (prod_lo, prod_hi, overflow) = {
                let prod = (val1 as i32) * (val2 as i32);
                let overflow = i16::try_from(prod).is_err();
                ((prod >> 16) as u16, prod as u16, overflow)
            };
            emulator.set_arithmetic_flags(prod_lo, overflow, overflow);
            emulator.set_reg(dst1, prod_lo);
            emulator.set_reg(dst2, prod_hi);
        },

        DivReg16<0x74, 30>(emulator) {
            let (dst1, dst2) = emulator.read_registers_operand();
            let (src, _) = emulator.read_registers_operand();
            let val1 = emulator.get_reg_mut(dst1);
            let val2 = emulator.get_reg_mut(src);
            if val2 == 0 {
                emulator.nasal_demons();
                return;
            }
            let quot = val1 / val2;
            let rem = val1 % val2;
            emulator.set_reg(dst1, quot);
            emulator.set_reg(dst2, rem);
        },
        DivImm8<0x75, 24>(emulator) {
            let (dst1, dst2) = emulator.read_registers_operand();
            let val1 = emulator.get_reg_mut(dst1);
            let val2 = emulator.read_from_pc::<u8, 1>() as u16;
            if val2 == 0 {
                emulator.nasal_demons();
                return;
            }
            let quot = val1 / val2;
            let rem = val1 % val2;
            emulator.set_reg(dst1, quot);
            emulator.set_reg(dst2, rem);
        },
        DivImm16<0x76, 30>(emulator) {
            let (dst1, dst2) = emulator.read_registers_operand();
            let val1 = emulator.get_reg_mut(dst1);
            let val2: u16 = emulator.read_from_pc();
            if val2 == 0 {
                emulator.nasal_demons();
                return;
            }
            let quot = val1 / val2;
            let rem = val1 % val2;
            emulator.set_reg(dst1, quot);
            emulator.set_reg(dst2, rem);
        },

        IdivReg16<0x64, 30>(emulator) {
            let (dst1, dst2) = emulator.read_registers_operand();
            let (src, _) = emulator.read_registers_operand();
            let val1: i16 = emulator.get_reg_mut(dst1) as i16;
            let val2: i16 = emulator.get_reg_mut(src) as i16;
            if val2 == 0 {
                emulator.nasal_demons();
                return;
            }
            let quot = val1.wrapping_div_euclid(val2);
            let rem = val1.wrapping_rem_euclid(val2);
            emulator.set_reg(dst1, quot as u16);
            emulator.set_reg(dst2, rem as u16);
        },
        IdivImm8<0x65, 24>(emulator) {
            let (dst1, dst2) = emulator.read_registers_operand();
            let val1: i16 = emulator.get_reg_mut(dst1) as i16;
            let val2: i16 = emulator.read_from_pc::<i8, 1>() as i16;
            if val2 == 0 {
                emulator.nasal_demons();
                return;
            }
            let quot = val1.wrapping_div_euclid(val2);
            let rem = val1.wrapping_rem_euclid(val2);
            emulator.set_reg(dst1, quot as u16);
            emulator.set_reg(dst2, rem as u16);
        },
        IdivImm16<0x66, 30>(emulator) {
            let (dst1, dst2) = emulator.read_registers_operand();
            let val1: i16 = emulator.get_reg_mut(dst1) as i16;
            let val2: i16 = emulator.read_from_pc();
            if val2 == 0 {
                emulator.nasal_demons();
                return;
            }
            let quot = val1.wrapping_div_euclid(val2);
            let rem = val1.wrapping_rem_euclid(val2);
            emulator.set_reg(dst1, quot as u16);
            emulator.set_reg(dst2, rem as u16);
        },

        Neg<0x7f, 3>(emulator) {
            let (reg, _) = emulator.read_registers_operand();
            let value = emulator.get_reg_mut(reg);
            let (result, carry) = value.overflowing_neg();
            let (_, overflow) = (value as i16).overflowing_neg();
            emulator.set_arithmetic_flags(result, carry, overflow);
            emulator.set_reg(reg, result);
        },

        Abs<0x8f, 3>(emulator) {
            let (reg, _) = emulator.read_registers_operand();
            let value: i16 = emulator.get_reg_mut(reg) as i16;
            let (result, overflow) = value.overflowing_abs();
            emulator.set_arithmetic_flags(result as u16, overflow, overflow);
            emulator.set_reg(reg, result as u16);
        },

        CmpReg16<0x54, 4>(emulator) {
            let (reg1, reg2) = emulator.read_registers_operand();
            let val1 = emulator.get_reg_mut(reg1);
            let val2 = emulator.get_reg_mut(reg2);
            let (sum, carry) = val1.overflowing_sub(val2);
            let (_, overflow) = (val1 as i16).overflowing_sub(val2 as i16);
            emulator.set_arithmetic_flags(sum, carry, overflow);
        },
        CmpImm8<0x55, 4>(emulator) {
            let (reg, _) = emulator.read_registers_operand();
            let val1 = emulator.get_reg_mut(reg);
            let val2 = emulator.read_from_pc::<i8, 1>() as u16;
            let (sum, carry) = val1.overflowing_sub(val2);
            let (_, overflow) = (val1 as i16).overflowing_sub(val2 as i16);
            emulator.set_arithmetic_flags(sum, carry, overflow);
        },
        CmpImm16<0x56, 4>(emulator) {
            let (reg, _) = emulator.read_registers_operand();
            let val1 = emulator.get_reg_mut(reg);
            let val2 = emulator.read_from_pc();
            let (sum, carry) = val1.overflowing_sub(val2);
            let (_, overflow) = (val1 as i16).overflowing_sub(val2 as i16);
            emulator.set_arithmetic_flags(sum, carry, overflow);
        },
    ]
}

instruction_category! {
    make_logical_instructions() => [
        Not<0x80, 3>(emulator) {
            let (reg, _) = emulator.read_registers_operand();
            let value = emulator.get_reg_mut(reg);
            let result = !value;
            emulator.set_reg(reg, result);
        },

        AndReg16<0x37, 4>(emulator) {
            let (dst, src) = emulator.read_registers_operand();
            let val1 = emulator.get_reg_mut(dst);
            let val2 = emulator.get_reg_mut(src);
            let result = val1 & val2;
            emulator.set_logical_flags(result);
            emulator.set_reg(dst, result);
        },
        AndImm8<0x36, 4>(emulator) {
            let (dst, _) = emulator.read_registers_operand();
            let val1 = emulator.get_reg_mut(dst);
            let val2 = emulator.read_from_pc::<u8, 1>() as u16;
            let result = val1 & val2;
            emulator.set_logical_flags(result);
            emulator.set_reg(dst, result);
        },
        AndImm16<0x46, 4>(emulator) {
            let (dst, _) = emulator.read_registers_operand();
            let val1 = emulator.get_reg_mut(dst);
            let val2: u16 = emulator.read_from_pc();
            let result = val1 & val2;
            emulator.set_logical_flags(result);
            emulator.set_reg(dst, result);
        },

        OrReg16<0x48, 4>(emulator) {
            let (dst, src) = emulator.read_registers_operand();
            let val1 = emulator.get_reg_mut(dst);
            let val2 = emulator.get_reg_mut(src);
            let result = val1 | val2;
            emulator.set_logical_flags(result);
            emulator.set_reg(dst, result);
        },
        OrImm8<0x47, 4>(emulator) {
            let (dst, _) = emulator.read_registers_operand();
            let val1 = emulator.get_reg_mut(dst);
            let val2 = emulator.read_from_pc::<u8, 1>() as u16;
            let result = val1 | val2;
            emulator.set_logical_flags(result);
            emulator.set_reg(dst, result);
        },
        OrImm16<0x57, 4>(emulator) {
            let (dst, _) = emulator.read_registers_operand();
            let val1 = emulator.get_reg_mut(dst);
            let val2: u16 = emulator.read_from_pc();
            let result = val1 | val2;
            emulator.set_logical_flags(result);
            emulator.set_reg(dst, result);
        },

        XorReg16<0x59, 4>(emulator) {
            let (dst, src) = emulator.read_registers_operand();
            let val1 = emulator.get_reg_mut(dst);
            let val2 = emulator.get_reg_mut(src);
            let result = val1 ^ val2;
            emulator.set_logical_flags(result);
            emulator.set_reg(dst, result);
        },
        XorImm8<0x58, 4>(emulator) {
            let (dst, _) = emulator.read_registers_operand();
            let val1 = emulator.get_reg_mut(dst);
            let val2 = emulator.read_from_pc::<u8, 1>() as u16;
            let result = val1 ^ val2;
            emulator.set_logical_flags(result);
            emulator.set_reg(dst, result);
        },
        XorImm16<0x68, 4>(emulator) {
            let (dst, _) = emulator.read_registers_operand();
            let val1 = emulator.get_reg_mut(dst);
            let val2: u16 = emulator.read_from_pc();
            let result = val1 ^ val2;
            emulator.set_logical_flags(result);
            emulator.set_reg(dst, result);
        },

        NandReg16<0x6a, 4>(emulator) {
            let (dst, src) = emulator.read_registers_operand();
            let val1 = emulator.get_reg_mut(dst);
            let val2 = emulator.get_reg_mut(src);
            let result = !(val1 & val2);
            emulator.set_logical_flags(result);
            emulator.set_reg(dst, result);
        },
        NandImm8<0x69, 4>(emulator) {
            let (dst, _) = emulator.read_registers_operand();
            let val1 = emulator.get_reg_mut(dst);
            let val2 = emulator.read_from_pc::<u8, 1>() as u16;
            let result = !(val1 & val2);
            emulator.set_logical_flags(result);
            emulator.set_reg(dst, result);
        },
        NandImm16<0x79, 4>(emulator) {
            let (dst, _) = emulator.read_registers_operand();
            let val1 = emulator.get_reg_mut(dst);
            let val2: u16 = emulator.read_from_pc();
            let result = !(val1 & val2);
            emulator.set_logical_flags(result);
            emulator.set_reg(dst, result);
        },

        NorReg16<0x7b, 4>(emulator) {
            let (dst, src) = emulator.read_registers_operand();
            let val1 = emulator.get_reg_mut(dst);
            let val2 = emulator.get_reg_mut(src);
            let result = !(val1 | val2);
            emulator.set_logical_flags(result);
            emulator.set_reg(dst, result);
        },
        NorImm8<0x7a, 4>(emulator) {
            let (dst, _) = emulator.read_registers_operand();
            let val1 = emulator.get_reg_mut(dst);
            let val2 = emulator.read_from_pc::<u8, 1>() as u16;
            let result = !(val1 | val2);
            emulator.set_logical_flags(result);
            emulator.set_reg(dst, result);
        },
        NorImm16<0x8a, 4>(emulator) {
            let (dst, _) = emulator.read_registers_operand();
            let val1 = emulator.get_reg_mut(dst);
            let val2: u16 = emulator.read_from_pc();
            let result = !(val1 | val2);
            emulator.set_logical_flags(result);
            emulator.set_reg(dst, result);
        },

        XnorReg16<0x8c, 4>(emulator) {
            let (dst, src) = emulator.read_registers_operand();
            let val1 = emulator.get_reg_mut(dst);
            let val2 = emulator.get_reg_mut(src);
            let result = !(val1 ^ val2);
            emulator.set_logical_flags(result);
            emulator.set_reg(dst, result);
        },
        XnorImm8<0x8b, 4>(emulator) {
            let (dst, _) = emulator.read_registers_operand();
            let val1 = emulator.get_reg_mut(dst);
            let val2 = emulator.read_from_pc::<u8, 1>() as u16;
            let result = !(val1 ^ val2);
            emulator.set_logical_flags(result);
            emulator.set_reg(dst, result);
        },
        XnorImm16<0x9b, 4>(emulator) {
            let (dst, _) = emulator.read_registers_operand();
            let val1 = emulator.get_reg_mut(dst);
            let val2: u16 = emulator.read_from_pc();
            let result = !(val1 ^ val2);
            emulator.set_logical_flags(result);
            emulator.set_reg(dst, result);
        },

        ShrReg16<0x9d, 4>(emulator) {
            let (dst, src) = emulator.read_registers_operand();
            let val1 = emulator.get_reg_mut(dst);
            let val2 = emulator.get_reg_mut(src);
            if val2 >= 16 {
                emulator.nasal_demons();
                return;
            }
            let result = val1 >> val2;
            emulator.set_logical_flags(result);
            emulator.set_reg(dst, result);
        },
        ShrImm8<0x9c, 4>(emulator) {
            let (dst, _) = emulator.read_registers_operand();
            let val1 = emulator.get_reg_mut(dst);
            let val2: u8 = emulator.read_from_pc();
            if val2 >= 16 {
                emulator.nasal_demons();
                return;
            }
            let result = val1 >> val2;
            emulator.set_logical_flags(result);
            emulator.set_reg(dst, result);
        },

        SarReg16<0xae, 4>(emulator) {
            let (dst, src) = emulator.read_registers_operand();
            let val1: i16 = emulator.get_reg_mut(dst) as i16;
            let val2 = emulator.get_reg_mut(src);
            if val2 >= 16 {
                emulator.nasal_demons();
                return;
            }
            let result = val1 >> val2;
            emulator.set_logical_flags(result as u16);
            emulator.set_reg(dst, result as u16);
        },
        SarImm8<0xad, 4>(emulator) {
            let (dst, _) = emulator.read_registers_operand();
            let val1: i16 = emulator.get_reg_mut(dst) as i16;
            let val2: u8 = emulator.read_from_pc();
            if val2 >= 16 {
                emulator.nasal_demons();
                return;
            }
            let result = val1 >> val2;
            emulator.set_logical_flags(result as u16);
            emulator.set_reg(dst, result as u16);
        },

        ShlReg16<0xbf, 4>(emulator) {
            let (dst, src) = emulator.read_registers_operand();
            let val1 = emulator.get_reg_mut(dst);
            let val2 = emulator.get_reg_mut(src);
            if val2 >= 16 {
                emulator.nasal_demons();
                return;
            }
            let result = val1 << val2;
            emulator.set_logical_flags(result);
            emulator.set_reg(dst, result);
        },
        ShlImm8<0xbe, 4>(emulator) {
            let (dst, _) = emulator.read_registers_operand();
            let val1 = emulator.get_reg_mut(dst);
            let val2: u8 = emulator.read_from_pc();
            if val2 >= 16 {
                emulator.nasal_demons();
                return;
            }
            let result = val1 << val2;
            emulator.set_logical_flags(result);
            emulator.set_reg(dst, result);
        },

        Ctz<0xf0, 4>(emulator) {
            let (dst, src) = emulator.read_registers_operand();
            let value = emulator.get_reg_mut(src);
            let result = value.trailing_zeros();
            emulator.set_reg(dst, result as u16);
        },

        Clz<0xe0, 4>(emulator) {
            let (dst, src) = emulator.read_registers_operand();
            let value = emulator.get_reg_mut(src);
            let result = value.leading_zeros();
            emulator.set_reg(dst, result as u16);
        },

        Popcnt<0xd0, 4>(emulator) {
            let (dst, src) = emulator.read_registers_operand();
            let value = emulator.get_reg_mut(src);
            let result = value.count_ones();
            emulator.set_reg(dst, result as u16);
        },

        RolReg16<0xc0, 4>(emulator) {
            let (dst, src) = emulator.read_registers_operand();
            let val1 = emulator.get_reg_mut(dst);
            let val2 = emulator.get_reg_mut(src);
            let result = val1.rotate_left(val2 as u32);
            emulator.set_reg(dst, result);
        },
        RolImm8<0xc1, 4>(emulator) {
            let (dst, _) = emulator.read_registers_operand();
            let val1 = emulator.get_reg_mut(dst);
            let val2: u8 = emulator.read_from_pc();
            let result = val1.rotate_left(val2 as u32);
            emulator.set_reg(dst, result);
        },

        RorReg16<0xb0, 4>(emulator) {
            let (dst, src) = emulator.read_registers_operand();
            let val1 = emulator.get_reg_mut(dst);
            let val2 = emulator.get_reg_mut(src);
            let result = val1.rotate_right(val2 as u32);
            emulator.set_reg(dst, result);
        },
        RorImm8<0xb1, 4>(emulator) {
            let (dst, _) = emulator.read_registers_operand();
            let val1 = emulator.get_reg_mut(dst);
            let val2: u8 = emulator.read_from_pc();
            let result = val1.rotate_right(val2 as u32);
            emulator.set_reg(dst, result);
        },

        Bswap<0xa0, 3>(emulator) {
            let (reg, _) = emulator.read_registers_operand();
            let value = emulator.get_reg_mut(reg);
            let result = value.swap_bytes();
            emulator.set_reg(reg, result);
        },

        PextReg16<0x90, 4>(emulator) {
            let (dst, src) = emulator.read_registers_operand();
            let val1 = emulator.get_reg_mut(dst);
            let val2 = emulator.get_reg_mut(src);
            let mut result = 0;
            for idx in (0..=15).rev() {
                if val2 & (1 << idx) != 0 {
                    result = result << 1 | ((val1 >> idx) & 1);
                }
            }
            emulator.set_reg(dst, result);
        },
        PextImm16<0x91, 4>(emulator) {
            let (dst, _) = emulator.read_registers_operand();
            let val1 = emulator.get_reg_mut(dst);
            let val2: u16 = emulator.read_from_pc();
            let mut result = 0;
            for idx in (0..=15).rev() {
                if val2 & (1 << idx) != 0 {
                    result = result << 1 | ((val1 >> idx) & 1);
                }
            }
            emulator.set_reg(dst, result);
        },
    ]
}

instruction_category! {
    make_control_flow_instrucions() => [
        CallReg<0xfe, 26>(emulator) {
            let (src, _) = emulator.read_registers_operand();
            let value = emulator.get_reg_mut(src);
            let pc_value = emulator.get_reg_mut(RegisterName::PC);
            let sp_value = emulator.get_reg_mut(RegisterName::SP);
            emulator.write_to_mem(sp_value, pc_value);
            emulator.set_reg(RegisterName::SP, sp_value.wrapping_add(2));
            emulator.set_reg(RegisterName::PC, value);
        },
        CallImm16<0xfd, 28>(emulator) {
            let value = emulator.read_from_pc();
            let pc_value = emulator.get_reg_mut(RegisterName::PC);
            let sp_value = emulator.get_reg_mut(RegisterName::SP);
            emulator.write_to_mem(sp_value, pc_value);
            emulator.set_reg(RegisterName::SP, sp_value.wrapping_add(2));
            emulator.set_reg(RegisterName::PC, value);
        },

        Ret<0xc3, 24>(emulator) {
            let sp_value = emulator.get_reg_mut(RegisterName::SP);
            emulator.set_reg(RegisterName::SP, sp_value.wrapping_sub(2));
            let value = emulator.peek_from_mem(sp_value.wrapping_sub(2));
            emulator.set_reg(RegisterName::PC, value);
        },
    ]
}

instruction_category! {
    make_misc_instructions() => [
        Nop<0x6e, 1>(emulator) {
            emulator.set_cpu_flag(CpuFlag::Sleep, true);
        },

        Op<0x6f, 1>(emulator) {
            if !emulator.get_cpu_flag(CpuFlag::Sleep) {
                emulator.nasal_demons();
            }
        },

        P<0x70, 1>(emulator) {
            if !emulator.get_cpu_flag(CpuFlag::Sleep) {
                emulator.nasal_demons();
                return;
            }
            emulator.set_cpu_flag(CpuFlag::Sleep, false);
        },

        Syscall<0x0f, 100>(_emulator) {},

        Reserved<0xff, 420>(emulator) {
            emulator.nasal_demons();
        },
    ]
}
