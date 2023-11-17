import re

class Assembler:
    REGS = ['PC', 'FL', 'CT', 'R0', 'R1', 'R2', 'R3', 'R4', 'R5', 'SP', 'TF', 'ZR', 'RR', 'TS', 'RE', 'AX']

    def __init__(self):
        self.instrs = {}
        self.labels = {}

    def add_instr(self, line):
        name, opcode, *fmt = line.split()
        opcode = int(opcode, 16)
        if name not in self.instrs:
            self.instrs[name] = []
        self.instrs[name].append((opcode, fmt))

    def parse_reg_arg(self, arg):
        return bytes([self.REGS.index(arg.upper())])

    def parse_reg_reg_arg(self, arg1, arg2):
        idx1 = self.REGS.index(arg1.upper())
        idx2 = self.REGS.index(arg2.upper())
        return bytes([idx1 + idx2 * 16])

    def parse_wtf(self, arg):
        if (match := re.match(r'\[(\w+)\]', arg)):
            base = match.group(1)
            return bytes([self.REGS.index(base.upper())])
        if (match := re.match(r'\[(\w+)\+(\w+)\]', arg)):
            base = match.group(1)
            disp = match.group(2)
            return bytes([self.REGS.index(base.upper()) | 0x40]) + int(disp, 0).to_bytes(2, 'little')
        if (match := re.match(r'\[(\w+)\+(\w+)\*(\w+)\]', arg)):
            base = match.group(1)
            index = match.group(2)
            scale = match.group(3)
            index_scale = self.REGS.index(index.upper()) | (min(int(scale, 0), 15)) << 4
            return bytes([self.REGS.index(base.upper()) | 0x80]) + index_scale.to_bytes(1, 'little')
        if (match := re.match(r'\[(\w+)\+(\w+)\*(\w+)\+(\w+)\]', arg)):
            base = match.group(1)
            index = match.group(2)
            scale = match.group(3)
            disp = match.group(4)
            index_scale = self.REGS.index(index.upper()) | (min(int(scale, 0), 15)) << 4
            return bytes([self.REGS.index(base.upper()) | 0xc0]) + index_scale.to_bytes(1, 'little') + int(disp, 0).to_bytes(2, 'little')

    def try_assemble_line(self, args, fmt):
        args_it = iter(args)
        code = b''
        for cur in fmt:
            if cur == '<reg>':
                arg = next(args_it)
                code += self.parse_reg_arg(arg)
            elif cur == '<reg><reg>':
                arg1 = next(args_it)
                arg2 = next(args_it)
                code += self.parse_reg_reg_arg(arg1, arg2)
            elif cur == '<imm8>':
                arg = next(args_it)
                if arg.startswith(':'):
                    code += self.labels[arg].to_bytes(1, 'little')
                else:
                    code += int(arg, 0).to_bytes(1, 'little')
            elif cur == '<imm16>':
                arg = next(args_it)
                if arg.startswith(':'):
                    code += self.labels[arg].to_bytes(2, 'little')
                else:
                    code += int(arg, 0).to_bytes(2, 'little')
            elif cur == '<mem>':
                arg = next(args_it)
                code += self.parse_wtf(arg)
            else:
                raise ValueError('???')
        return code

    def assemble_line(self, asm):
        name, *args = asm.split()
        args = [arg.rstrip(',') for arg in args]
        for opcode, fmt in self.instrs[name]:
            try:
                code = bytes([opcode]) + self.try_assemble_line(args, fmt)
                if code:
                    return code
            except Exception as e:
                pass
        else:
            raise ValueError(f'Invalid line: {asm}')

    def assemble(self, asms):
        code = b''
        for asm in asms.splitlines():
            asm = asm.strip()
            if not asm or asm.startswith('//'):
                continue
            if asm.startswith(':'):
                self.labels[asm] = len(code)
                continue
            code += self.assemble_line(asm)
        return code


assembler = Assembler()
for instr in open('instrs.txt'):
    assembler.add_instr(instr)


print(assembler.assemble('''
// this is a comment
mov8 ax, 0x0
syscall
mov16 r0, ax

// i love x86
load16 r0, [r0]
store16 [r0+0x123], r0
load16 r1, [r0+ax*4]
store16 [r0+ax*4+0x123], r1

cmp16 r0, r1
cmovbe ax, r0

:label1
nop
op
p
mov16 pc, :label1
''').hex())
# remember to pad code to 65536 bytes
