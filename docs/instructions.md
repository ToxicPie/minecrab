# Minecrab Instruction Set Architecture Documentation

## Overview

This is a 16-bit architecture. All registers and memory addresses are 16-bit. Values are stored in little-endian.

The machine has 65536 bytes of memory, which means that every virtual address is valid. The stack grows up from the address 0x0, and the user is responsible for not letting the stack overflow into whichever parts of the memory they're using.

The bytecode also takes up 65536 bytes, so no matter where the PC is, it will always have code to execute. Memory and bytecode are in separate address spaces, which means that a program cannot directly access its bytecode.

Instructions have variable lengths. All values from 0 to 255 are valid opcodes, so that no matter where the program counter lands on, it can and will always continue to execute normally. Each instruction takes a fixed amount of cycles to execute, and the bytecode of a program can execute for some maximum number of cycles before it is paused and execution is switched to another program.

## Registers

|Index|Name|Description|Notes|
|:---:|:--:|:----------|:--|
|0x00 |PC  |Program Counter|The address of the next instruction. To change control flow, write to this register directly.|
|0x01 |FL  |CPU Flags|Stores CPU flags in some of its bits.|
|0x02 |CT  |Counter|Write behaves normally. Increments by 1 after every read.|
|0x03 |R0  |General-Purpose Register 0|Does what you think it does.|
|0x04 |R1  |General-Purpose Register 1|Does what you think it does.|
|0x05 |R2  |General-Purpose Register 2|Does what you think it does.|
|0x06 |R3  |General-Purpose Register 3|Does what you think it does.|
|0x07 |R4  |General-Purpose Register 4|Does what you think it does.|
|0x08 |R5  |General-Purpose Register 5|Does what you think it does.|
|0x09 |SP  |Stack Pointer|Stores the top (exclusive) of the stack. The stack grows up from 0x0.|
|0x0A |TF  |TheFuck|Always returns 0x1337 when read. Writing to this is no-op.|
|0x0B |ZR  |Zero Register|Always returns 0x0 when read. Writing to this is no-op.|
|0x0C |RR  |Random Register|Returns a random 16-bit value when read. Writing to this is no-op.|
|0x0D |TS  |Timestamp|Incremented by 1 at the beginning of each tick.|
|0x0E |RE  |Reversed|This register is a hardware bug. Every value read from it will have its bits reversed.|
|0x0F |AX  |Accumulator|General-purpose register. Used for the system call number and return value.

### CPU Flags

The corresponding bit in the `FL` register is updated when a CPU flag is changed. You can modify this register like a regular one, but it may cause weird behavior.

The 0-th to 4-th bits represent the Zero (ZF), Carry (CF), Overflow (OF), Sign (SF), and Sleep (NF) flags respectively. The other bits are unused.

#### Logical Flags

The logical flags are Zero and Sign indicating whether or not a result is zero or is negative.

#### Arithmetic Flags

The arithmetic flags are the logical flags, plus Carry and Overflow indicating unsigned carry/borrow and signed overflow in an arithmetic operation.

#### Sleep Flag

The Sleep flag indicates that the emulator is in the middle of a `nop` instruction chain.

The Sleep flag is set after a `nop` instruction. While the Sleep flag is set:
- If the current opcode is `6f` (`op`), the processor does nothing and PC += 1.
- If the current opcode is `70` (`p`), the processor does nothing, PC += 1, then resets the Sleep flag.
- Otherwise, it is undefined behavior.

## Encoding

### Register

A register operand is encoded as one byte. The lower 4 bits represent the register's index.

If the next operand is also a register, then they are encoded in the same byte. The lower 4 bits is the first operand and the upper 4 bits is the second.

### Immediate Value

An immediate value is encoded as the value in little-endian.

### Memory Address

A memory address can be encoded in one of the following ways:

1. `base`

    <table style="text-align:center">
        <tr>
            <th scope=row>Byte</th>
            <td colspan=8>0</td>
        </tr>
        <tr>
            <th scope=row>Bit</th>
            <td>7</td>
            <td>6</td><td>5</td><td>4</td><td>3</td><td>2</td><td>1</td><td>0</td>
        </tr>
        <tr>
            <th scope=row>Usage</th>
            <td colspan=1>0</td>
            <td colspan=1>0</td>
            <td colspan=2>-</td>
            <td colspan=4>Base (register)</td>
        </tr>
    </table>

2. `base + displacement`

    <table style="text-align:center">
        <tr>
            <th scope=row>Byte</th>
            <td colspan=8>0</td>
            <td colspan=8>1</td>
            <td colspan=8>2</td>
        </tr>
        <tr>
            <th scope=row>Bit</th>
            <td>7</td>
            <td>6</td><td>5</td><td>4</td><td>3</td><td>2</td><td>1</td><td>0</td>
            <td>7</td>
            <td>6</td><td>5</td><td>4</td><td>3</td><td>2</td><td>1</td><td>0</td>
            <td>7</td>
            <td>6</td><td>5</td><td>4</td><td>3</td><td>2</td><td>1</td><td>0</td>
        </tr>
        <tr>
            <th scope=row>Usage</th>
            <td colspan=1>0</td>
            <td colspan=1>1</td>
            <td colspan=2>-</td>
            <td colspan=4>Base (register)</td>
            <td colspan=16>Displacement (16-bit immediate value)</td>
        </tr>
    </table>

3. `base + index * scale`

    <table style="text-align:center">
        <tr>
            <th scope=row>Byte</th>
            <td colspan=8>0</td>
            <td colspan=8>1</td>
        </tr>
        <tr>
            <th scope=row>Bit</th>
            <td>7</td>
            <td>6</td><td>5</td><td>4</td><td>3</td><td>2</td><td>1</td><td>0</td>
            <td>7</td>
            <td>6</td><td>5</td><td>4</td><td>3</td><td>2</td><td>1</td><td>0</td>
        </tr>
        <tr>
            <th scope=row>Usage</th>
            <td colspan=1>1</td>
            <td colspan=1>0</td>
            <td colspan=2>-</td>
            <td colspan=4>Base (register)</td>
            <td colspan=4>Scale (4-bit)</td>
            <td colspan=4>Index (register)</td>
        </tr>
    </table>

4. `base + index * scale + displacement`

    <table style="text-align:center">
        <tr>
            <th scope=row>Byte</th>
            <td colspan=8>0</td>
            <td colspan=8>1</td>
            <td colspan=8>2</td>
            <td colspan=8>3</td>
        </tr>
        <tr>
            <th scope=row>Bit</th>
            <td>7</td>
            <td>6</td><td>5</td><td>4</td><td>3</td><td>2</td><td>1</td><td>0</td>
            <td>7</td>
            <td>6</td><td>5</td><td>4</td><td>3</td><td>2</td><td>1</td><td>0</td>
            <td>7</td>
            <td>6</td><td>5</td><td>4</td><td>3</td><td>2</td><td>1</td><td>0</td>
            <td>7</td>
            <td>6</td><td>5</td><td>4</td><td>3</td><td>2</td><td>1</td><td>0</td>
        </tr>
        <tr>
            <th scope=row>Usage</th>
            <td colspan=1>1</td>
            <td colspan=1>1</td>
            <td colspan=2>-</td>
            <td colspan=4>Base (register)</td>
            <td colspan=4>Scale (4-bit)</td>
            <td colspan=4>Index (register)</td>
            <td colspan=16>Displacement (16-bit immediate value)</td>
        </tr>
    </table>

In the above, `base` and `index` are values stored in registers, `displacement` is a 16-bit constant value stored in little-endian, and `scale` is a constant 4-bit value. When computing the address, `scale` is rounded up to the nearest power of 2, so the actual value can only be 1, 2, 4, 8, or 16.

## Opcode Table

<table style="text-align:center">
<tr>
    <td></td>
    <td>0</td><td>1</td><td>2</td><td>3</td><td>4</td><td>5</td><td>6</td><td>7</td>
    <td>8</td><td>9</td><td>a</td><td>b</td><td>c</td><td>d</td><td>e</td><td>f</td>
</tr>
<tr>
    <td>0</td>
    <td>00</td><td>01</td><td>02</td><td>03</td><td>04</td><td>05</td><td>06</td><td>07</td>
    <td>08</td><td>09</td><td>0a</td><td>0b</td><td>0c</td><td>0d</td><td>0e</td><td>Syscall</td>
</tr>
<tr>
    <td>1</td>
    <td>10</td><td>11</td><td>12</td><td>13</td><td>14</td><td>15</td><td>16</td><td>17</td>
    <td>18</td><td>19</td><td>1a</td><td>1b</td><td>1c</td><td>1d</td><td>1e</td><td>1f</td>
</tr>
<tr>
    <td>2</td>
    <td>Mov</td><td>Mov</td><td>Mov</td><td>Mov</td><td>Load</td><td>Load</td><td>Store</td><td>Store</td>
    <td>Store</td><td>Store</td><td>2a</td><td>2b</td><td>2c</td><td>2d</td><td>2e</td><td>2f</td>
</tr>
<tr>
    <td>3</td>
    <td>30</td><td>31</td><td>32</td><td>33</td><td>34</td><td>35</td><td>And</td><td>And</td>
    <td>38</td><td>39</td><td>3a</td><td>3b</td><td>3c</td><td>3d</td><td>3e</td><td>3f</td>
</tr>
<tr>
    <td>4</td>
    <td>40</td><td>41</td><td>42</td><td>43</td><td>44</td><td>45</td><td>And</td><td>Or</td>
    <td>Or</td><td>49</td><td>4a</td><td>4b</td><td>4c</td><td>4d</td><td>4e</td><td>4f</td>
</tr>
<tr>
    <td>5</td>
    <td>Push</td><td>Push</td><td>Push</td><td>Push</td><td>Cmp</td><td>Cmp</td><td>Cmp</td><td>Or</td>
    <td>Xor</td><td>Xor</td><td>5a</td><td>5b</td><td>5c</td><td>5d</td><td>5e</td><td>5f</td>
</tr>
<tr>
    <td>6</td>
    <td>Pop</td><td>Pop</td><td>Sex</td><td>63</td><td>Idiv</td><td>Idiv</td><td>Idiv</td><td>67</td>
    <td>Xor</td><td>Nand</td><td>Nand</td><td>6b</td><td>6c</td><td>6d</td><td>Nop</td><td>Op</td>
</tr>
<tr>
    <td>7</td>
    <td>P</td><td>Memcpy</td><td>72</td><td>73</td><td>Div</td><td>Div</td><td>Div</td><td>77</td>
    <td>78</td><td>Nand</td><td>Nor</td><td>Nor</td><td>7c</td><td>7d</td><td>7e</td><td>Neg</td>
</tr>
<tr>
    <td>8</td>
    <td>Not</td><td>Memset</td><td>Memset</td><td>83</td><td>Imul</td><td>Imul</td><td>Imul</td><td>87</td>
    <td>88</td><td>89</td><td>Nor</td><td>Xnor</td><td>Xnor</td><td>Lea</td><td>8e</td><td>Abs</td>
</tr>
<tr>
    <td>9</td>
    <td>Pext</td><td>Pext</td><td>Xchg</td><td>93</td><td>Mullo</td><td>Mullo</td><td>Mullo</td><td>97</td>
    <td>98</td><td>99</td><td>9a</td><td>Xnor</td><td>Shr</td><td>Shr</td><td>9e</td><td>9f</td>
</tr>
<tr>
    <td>a</td>
    <td>Bswap</td><td>a1</td><td>a2</td><td>a3</td><td>Mul</td><td>Mul</td><td>Mul</td><td>a7</td>
    <td>a8</td><td>a9</td><td>aa</td><td>ab</td><td>ac</td><td>Sar</td><td>Sar</td><td>af</td>
</tr>
<tr>
    <td>b</td>
    <td>Ror</td><td>Ror</td><td>b2</td><td>b3</td><td>Sub</td><td>Sub</td><td>Sub</td><td>b7</td>
    <td>b8</td><td>b9</td><td>ba</td><td>bb</td><td>bc</td><td>bd</td><td>Shl</td><td>Shl</td>
</tr>
<tr>
    <td>c</td>
    <td>Rol</td><td>Rol</td><td>c2</td><td>Ret</td><td>Add</td><td>Add</td><td>Add</td><td>c7</td>
    <td>c8</td><td>c9</td><td>ca</td><td>cb</td><td>cc</td><td>cd</td><td>ce</td><td>cf</td>
</tr>
<tr>
    <td>d</td>
    <td>Popcnt</td><td>d1</td><td>d2</td><td>d3</td><td>d4</td><td>d5</td><td>Cmova</td><td>Cmovae</td>
    <td>Cmovb</td><td>Cmovbe</td><td>Cmove</td><td>Cmovne</td><td>Cmovg</td><td>Cmovge</td><td>Cmovl</td><td>Cmovle</td>
</tr>
<tr>
    <td>e</td>
    <td>Clz</td><td>e1</td><td>e2</td><td>e3</td><td>e4</td><td>e5</td><td>Cmova</td><td>Cmovae</td>
    <td>Cmovb</td><td>Cmovbe</td><td>Cmove</td><td>Cmovne</td><td>Cmovg</td><td>Cmovge</td><td>Cmovl</td><td>Cmovle</td>
</tr>
<tr>
    <td>f</td>
    <td>Ctz</td><td>f1</td><td>f2</td><td>f3</td><td>f4</td><td>f5</td><td>f6</td><td>f7</td>
    <td>f8</td><td>f9</td><td>fa</td><td>fb</td><td>fc</td><td>Call</td><td>Call</td><td>ff</td>
</tr>
</table>

## Assignment Instructions

### Mov

Sets the value of a register to that of another register, or to an immediate value.

- `mov8 <dst_reg> <src_reg>`
    - Sets the lower 8 bits and zeroes the upper 8 bits of the destination.
    - Encoding: `23 <reg><reg>`
    - Latency: 3 cycles
- `mov16 <dst_reg> <src_reg>`
    - Encoding: `22 <reg><reg>`
    - Latency: 3 cycles
- `mov8 <dst_reg> <imm8>`
    - The upper 8 bits become 0.
    - Encoding: `21 <reg> <imm8>`
    - Latency: 3 cycles
- `mov16 <dst_reg> <imm16>`
    - Encoding: `20 <reg> <imm16>`
    - Latency: 4 cycles

### Load

Loads a value from a memory address into a register.

- `load8 <dst_reg> <addr>`
    - The upper 8 bits become 0.
    - Encoding: `25 <reg> <mem>`
    - Latency: 24 cycles
- `load16 <dst_reg> <addr>`
    - Encoding: `24 <reg> <mem>`
    - Latency: 28 cycles

### Store

Stores the value in a register into a memory address.

- `store8 <dst_addr> <src_reg>`
    - Stores the lower 8 bits of `src_reg`.
    - Encoding: `29 <mem> <reg>`
    - Latency: 22 cycles
- `store16 <dst_addr> <src_reg>`
    - Encoding: `28 <mem> <reg>`
    - Latency: 26 cycles
- `store8 <dst_addr> <imm8>`
    - Encoding: `27 <mem> <imm8>`
    - Latency: 24 cycles
- `store16 <dst_addr> <imm16>`
    - Encoding: `26 <mem> <imm16>`
    - Latency: 28 cycles

### Lea

Load a memory address into a register.

- `lea <reg> <addr>`
    - Encoding: `8d <reg> <mem>`
    - Latency: 5 cycles

### Xchg

Exchanges the values of two registers.

- `xchg <reg1> <reg2>`
    - Encoding: `92 <reg><reg>`
    - Latency: 3 cycles

### Sex

Converts the lower 8 bits of a register to 16 bits, with sign extension.

- `sex16 <reg>`
    - Encoding: `62 <reg>`
    - Latency: 3 cycles

### Cmov

Conditional move based on CPU flags.

- `cmova <dst_reg> <src_reg>`
    - Mov if above (!CF && !ZF)
    - Encoding: `d6 <reg><reg>`
    - Latency: 8 cycles
- `cmova16 <dst_reg> <imm16>`
    - Mov if above (!CF && !ZF)
    - Encoding: `e6 <reg> <imm16>`
    - Latency: 9 cycles
- `cmovae <dst_reg> <src_reg>`
    - Mov if above or equal (!CF)
    - Encoding: `d7 <reg><reg>`
    - Latency: 8 cycles
- `cmovae16 <dst_reg> <imm16>`
    - Mov if above or equal (!CF)
    - Encoding: `e7 <reg> <imm16>`
    - Latency: 9 cycles
- `cmovb <dst_reg> <src_reg>`
    - Mov if below (CF)
    - Encoding: `d8 <reg><reg>`
    - Latency: 8 cycles
- `cmovb16 <dst_reg> <imm16>`
    - Mov if below (CF)
    - Encoding: `e8 <reg> <imm16>`
    - Latency: 9 cycles
- `cmovbe <dst_reg> <src_reg>`
    - Mov if below or equal (CF || ZF)
    - Encoding: `d9 <reg><reg>`
    - Latency: 8 cycles
- `cmovbe16 <dst_reg> <imm16>`
    - Mov if below or equal (CF || ZF)
    - Encoding: `e9 <reg> <imm16>`
    - Latency: 9 cycles
- `cmove <dst_reg> <src_reg>`
    - Mov if equal (ZF)
    - Encoding: `da <reg><reg>`
    - Latency: 8 cycles
- `cmove16 <dst_reg> <imm16>`
    - Mov if equal (ZF)
    - Encoding: `ea <reg> <imm16>`
    - Latency: 9 cycles
- `cmovne <dst_reg> <src_reg>`
    - Mov if not equal (!ZF)
    - Encoding: `db <reg><reg>`
    - Latency: 8 cycles
- `cmovne16 <dst_reg> <imm16>`
    - Mov if not equal (!ZF)
    - Encoding: `eb <reg> <imm16>`
    - Latency: 9 cycles
- `cmovg <dst_reg> <src_reg>`
    - Mov if greater (!ZF && SF == OF)
    - Encoding: `dc <reg><reg>`
    - Latency: 8 cycles
- `cmovg16 <dst_reg> <imm16>`
    - Mov if greater (!ZF && SF == OF)
    - Encoding: `ec <reg> <imm16>`
    - Latency: 9 cycles
- `cmovge <dst_reg> <src_reg>`
    - Mov if greater or equal (SF == OF)
    - Encoding: `dd <reg><reg>`
    - Latency: 8 cycles
- `cmovge16 <dst_reg> <imm16>`
    - Mov if greater or equal (SF == OF)
    - Encoding: `ed <reg> <imm16>`
    - Latency: 9 cycles
- `cmovl <dst_reg> <src_reg>`
    - Mov if less (SF != OF)
    - Encoding: `de <reg><reg>`
    - Latency: 8 cycles
- `cmovl16 <dst_reg> <imm16>`
    - Mov if less (SF != OF)
    - Encoding: `ee <reg> <imm16>`
    - Latency: 9 cycles
- `cmovle <dst_reg> <src_reg>`
    - Mov if less or equal (ZF || SF != OF)
    - Encoding: `df <reg><reg>`
    - Latency: 8 cycles
- `cmovle16 <dst_reg> <imm16>`
    - Mov if less or equal (ZF || SF != OF)
    - Encoding: `ef <reg> <imm16>`
    - Latency: 9 cycles

### Push

Equivalent to `store SP, <value>` then `add SP, <size>`.

- `push8 <reg>`
    - Pushes the lower 8 bits.
    - Encoding: `50 <src_reg>`
    - Latency: 24 cycles
- `push16 <reg>`
    - Encoding: `51 <src_reg>`
    - Latency: 28 cycles
- `push8 <imm8>`
    - Encoding: `52 <imm8>`
    - Latency: 26 cycles
- `push16 <imm16>`
    - Encoding: `53 <imm16>`
    - Latency: 30 cycles

### Pop

Equivalent to `sub SP, <size>` then `load <reg>, SP`.

- `pop8 <reg>`
    - the upper 8 bits become 0.
    - Encoding: `60 <dst_reg>`
    - Latency: 26 cycles
- `pop16 <reg>`
    - Encoding: `61 <dst_reg>`
    - Latency: 30 cycles

### Memcpy

Copies `n + 1` bytes of memory from the address range `[src, src + n]` to the range `[dst, dst + n]`, wrapping around at the memory address boundary, where `n` is an 8-bit unsigned integer. The memory ranges must not overlap, otherwise it is undefined behavior.

Notice that the number of bytes copied is `n + 1`.

- `memcpy <n> <dst_addr> <src_addr>`
    - `n` is the lower 8 bits of `reg`.
    - Encoding: `71 <reg> <mem> <mem>`
    - Latency: 512 cycles

### Memset

Sets every byte in memory in the address range `[addr, addr + n]` to a byte `b`, wrapping around at the memory address boundary, where `n` is an 8-bit unsigned integer.

Notice that the number of bytes written is `n + 1`.

- `memcpy <n> <addr> <b>`
    - `n` and `b` are the lower 8 bits of corresponding `reg`s.
    - Encoding: `81 <reg> <mem> <reg>`
    - Latency: 384 cycles
- `memcpy <n> <addr> <b>`
    - `n` is the lower 8 bits of `reg`.
    - Encoding: `82 <reg> <mem> <imm8>`
    - Latency: 384 cycles

## Arithmetic Instructions

### Add

Adds two integers. `dst += src`. Modifies arithmetic flags according to `dst`.

- `add16 <dst_reg> <src_reg>`
    - Encoding: `c4 <reg><reg>`
    - Latency: 4 cycles
- `add8 <dst_reg> <imm8>`
    - The second operand is sign extended.
    - Encoding: `c5 <reg> <imm8>`
    - Latency: 4 cycles
- `add16 <dst_reg> <imm16>`
    - Encoding: `c6 <reg> <imm16>`
    - Latency: 4 cycles

### Sub

Subtracts two integers. `dst -= src`. Modifies arithmetic flags according to `dst`.

- `sub16 <dst_reg> <src_reg>`
    - Encoding: `b4 <reg><reg>`
    - Latency: 4 cycles
- `sub8 <dst_reg> <imm8>`
    - The second operand is sign extended.
    - Encoding: `b5 <reg> <imm8>`
    - Latency: 4 cycles
- `sub16 <dst_reg> <imm16>`
    - Encoding: `b6 <reg> <imm16>`
    - Latency: 4 cycles

### Mul

Multiplies two unsigned integers. `dst2` and `dst1` are the upper and lower 16 bits of `dst1 * src`. Modifies arithmetic flags according to `dst1`.

- `mul16 <dst1_reg> <dst2_reg> <src_reg>`
    - Encoding: `a4 <reg><reg> <reg>`
    - Latency: 18 cycles
- `mul8 <dst1_reg> <dst2_reg> <imm8>`
    - The second operand is zero extended.
    - Encoding: `a5 <reg><reg> <imm8>`
    - Latency: 16 cycles
- `mul16 <dst1_reg> <dst2_reg> <imm16>`
    - Encoding: `a6 <reg><reg> <imm16>`
    - Latency: 18 cycles

### Mullo

Multiplies two integers, modulo 2^16. `dst *= src`. Modifies arithmetic flags according to `dst1`.

- `mullo16 <dst_reg> <src_reg>`
    - Encoding: `94 <reg><reg>`
    - Latency: 12 cycles
- `mullo8 <dst_reg> <imm8>`
    - The second operand is zero extended.
    - Encoding: `95 <reg> <imm8>`
    - Latency: 10 cycles
- `mullo16 <dst_reg> <imm16>`
    - Encoding: `96 <reg> <imm16>`
    - Latency: 12 cycles

### Imul

Multiplies two signed integers. `dst2` and `dst1` are the upper and lower 16 bits of `dst1 * src`. Modifies arithmetic flags according to `dst1`.

- `imul16 <dst1_reg> <dst2_reg> <src_reg>`
    - Encoding: `84 <reg><reg> <reg>`
    - Latency: 18 cycles
- `imul8 <dst1_reg> <dst2_reg> <imm8>`
    - The second operand is sign extended.
    - Encoding: `85 <reg><reg> <imm8>`
    - Latency: 16 cycles
- `imul16 <dst1_reg> <dst2_reg> <imm16>`
    - Encoding: `86 <reg><reg> <imm16>`
    - Latency: 18 cycles

### Div

Divides two unsigned integers. `dst1 = dst1 / src` and `dst2 = dst1 % src`. It is undefined behavior to divide by zero.

- `div16 <dst1_reg> <dst2_reg> <src_reg>`
    - Encoding: `74 <reg><reg> <reg>`
    - Latency: 30 cycles
- `div8 <dst1_reg> <dst2_reg> <imm8>`
    - Encoding: `75 <reg><reg> <imm8>`
    - Latency: 24 cycles
- `div16 <dst1_reg> <dst2_reg> <imm16>`
    - Encoding: `76 <reg><reg> <imm16>`
    - Latency: 30 cycles

### Idiv

Divides two signed integers. `dst1 = dst1 / src` and `dst2 = dst1 % src`. The remainder is chosen such that `0 <= dst2 < abs(src)`. The result is taken mod 2^16 in case of overflow (only happens for `-32768 / -1`). It is undefined behavior to divide by zero.

- `idiv16 <dst1_reg> <dst2_reg> <src_reg>`
    - Encoding: `64 <reg><reg> <reg>`
    - Latency: 30 cycles
- `idiv8 <dst1_reg> <dst2_reg> <imm8>`
    - Encoding: `65 <reg><reg> <imm8>`
    - Latency: 26 cycles
- `idiv16 <dst1_reg> <dst2_reg> <imm16>`
    - Encoding: `66 <reg><reg> <imm16>`
    - Latency: 30 cycles

### Neg

Negates (subtracts from 0) a signed integer. If the input is -2^15, returns itself. Modifies arithmetic flags according to `reg`.

- `neg <reg>`
    - Encoding: `7f <reg>`
    - Latency: 3 cycles

### Abs

Computes the absolute value (subtracts from 0 if negative) of a signed integer. If the input is -2^15, returns itself. Modifies arithmetic flags according to `reg`.

- `abs <reg>`
    - Encoding: `8f <reg>`
    - Latency: 3 cycles

### Cmp

Sets the flags according to the result of `dst - src`.

- `cmp16 <dst_reg> <src_reg>`
    - Encoding: `54 <reg><reg>`
    - Latency: 4 cycles
- `cmp8 <dst_reg> <imm8>`
    - The second operand is sign extended.
    - Encoding: `55 <reg> <imm8>`
    - Latency: 4 cycles
- `cmp16 <dst_reg> <imm16>`
    - Encoding: `56 <reg> <imm16>`
    - Latency: 4 cycles

## Logical Instructions

### Not

Computes the bitwise not of an integer.

- `not <reg>`
    - Encoding: `80 <reg>`
    - Latency: 3 cycles

### And

Computes the bitwise AND of two integers. `dst &= src`. Modifies logical flags according to `dst`.

- `and16 <dst_reg> <src_reg>`
    - Encoding: `37 <reg><reg>`
    - Latency: 4 cycles
- `and8 <dst_reg> <imm8>`
    - The second operand is zero extended.
    - Encoding: `36 <reg> <imm8>`
    - Latency: 4 cycles
- `and16 <dst_reg> <imm16>`
    - Encoding: `46 <reg> <imm16>`
    - Latency: 4 cycles

### Or

Computes the bitwise OR of two integers. `dst |= src`. Modifies logical flags according to `dst`.

- `or16 <dst_reg> <src_reg>`
    - Encoding: `48 <reg><reg>`
    - Latency: 4 cycles
- `or8 <dst_reg> <imm8>`
    - The second operand is zero extended.
    - Encoding: `47 <reg> <imm8>`
    - Latency: 4 cycles
- `or16 <dst_reg> <imm16>`
    - Encoding: `57 <reg> <imm16>`
    - Latency: 4 cycles

### Xor

Computes the bitwise XOR of two integers. `dst ^= src`. Modifies logical flags according to `dst`.

- `xor16 <dst_reg> <src_reg>`
    - Encoding: `59 <reg><reg>`
    - Latency: 4 cycles
- `xor8 <dst_reg> <imm8>`
    - The second operand is zero extended.
    - Encoding: `58 <reg> <imm8>`
    - Latency: 4 cycles
- `xor16 <dst_reg> <imm16>`
    - Encoding: `68 <reg> <imm16>`
    - Latency: 4 cycles

### Nand

Computes the bitwise NAND of two integers. `dst = ~(dst & src)`. Modifies logical flags according to `dst`.

- `nand16 <dst_reg> <src_reg>`
    - Encoding: `6a <reg><reg>`
    - Latency: 4 cycles
- `nand8 <dst_reg> <imm8>`
    - The second operand is zero extended.
    - Encoding: `69 <reg> <imm8>`
    - Latency: 4 cycles
- `nand16 <dst_reg> <imm16>`
    - Encoding: `79 <reg> <imm16>`
    - Latency: 4 cycles

### Nor

Computes the bitwise NOR of two integers. `dst = ~(dst | src)`. Modifies logical flags according to `dst`.

- `nor16 <dst_reg> <src_reg>`
    - Encoding: `7b <reg><reg>`
    - Latency: 4 cycles
- `nor8 <dst_reg> <imm8>`
    - The second operand is zero extended.
    - Encoding: `7a <reg> <imm8>`
    - Latency: 4 cycles
- `nor16 <dst_reg> <imm16>`
    - Encoding: `8a <reg> <imm16>`
    - Latency: 4 cycles

### Xnor

Computes the bitwise XNOR of two integers. `dst = ~(dst ^ src)`. Modifies logical flags according to `dst`.

- `xnor16 <dst_reg> <src_reg>`
    - Encoding: `8c <reg><reg>`
    - Latency: 4 cycles
- `xnor8 <dst_reg> <imm8>`
    - The second operand is zero extended.
    - Encoding: `8b <reg> <imm8>`
    - Latency: 4 cycles
- `xnor16 <dst_reg> <imm16>`
    - Encoding: `9b <reg> <imm16>`
    - Latency: 4 cycles

### Shr

Performs a right shift on an unsigned integer. `dst >>= src`. It is undefined behavior to shift more than 15 bits. Modifies logical flags according to `dst`.

- `shr <dst_reg> <src_reg>`
    - Encoding: `9d <reg><reg>`
    - Latency: 4 cycles
- `shr <dst_reg> <imm8>`
    - Encoding: `9c <reg> <imm8>`
    - Latency: 4 cycles

### Sar

Performs a right shift on a signed integer. `dst >>= src`. It is undefined behavior to shift more than 15 bits. Modifies logical flags according to `dst`.

- `sar <dst_reg> <src_reg>`
    - Encoding: `ae <reg><reg>`
    - Latency: 4 cycles
- `sar <dst_reg> <imm8>`
    - Encoding: `ad <reg> <imm8>`
    - Latency: 4 cycles

### Shl

Performs a left shift on an integer. `dst <<= src`. It is undefined behavior to shift more than 15 bits. Modifies logical flags according to `dst`.

- `shl <dst_reg> <src_reg>`
    - Encoding: `bf <reg><reg>`
    - Latency: 4 cycles
- `shl <dst_reg> <imm8>`
    - Encoding: `be <reg> <imm8>`
    - Latency: 4 cycles

### Ctz

Counts the number of trailing zero bits of an integer.

- `ctz <dst_reg> <src_reg>`
    - Encoding: `f0 <reg><reg>`
    - Latency: 4 cycles

### Clz

Counts the number of leading zero bits of an integer.

- `clz <dst_reg> <src_reg>`
    - Encoding: `e0 <reg><reg>`
    - Latency: 4 cycles

### Popcnt

Counts the number of one bits in an integer.

- `popcnt <dst_reg> <src_reg>`
    - Encoding: `d0 <reg><reg>`
    - Latency: 4 cycles

### Rol

Performs a bit rotation to the left. Rotating by more than 15 bits is equivalent to rotating by the shift value mod 16.

- `rol <dst_reg> <src_reg>`
    - Encoding: `c0 <reg><reg>`
    - Latency: 4 cycles
- `rol <dst_reg> <imm8>`
    - Encoding: `c1 <reg> <imm8>`
    - Latency: 4 cycles

### Ror

Performs a bit rotation to the right. Rotating by more than 15 bits is equivalent to rotating by the shift value mod 16.

- `ror <dst_reg> <src_reg>`
    - Encoding: `b0 <reg><reg>`
    - Latency: 4 cycles
- `ror <dst_reg> <imm8>`
    - Encoding: `b1 <reg> <imm8>`
    - Latency: 4 cycles

### Bswap

Swaps the upper and lower 8 bits of a 16-bit integer.

- `bswap <reg>`
    - Encoding: `a0 <reg>`
    - Latency: 3 cycles

### Pext

Extract the bits of the first operand at positions indicated by a mask (the second operand), and place them in the contiguous lower bit positions in the first operand. The other bits are set to 0.

- `pext <dst_reg> <src_reg>`
    - Encoding: `90 <reg><reg>`
    - Latency: 15 cycles
- `pext <dst_reg> <imm16>`
    - Encoding: `91 <reg> <imm16>`
    - Latency: 15 cycles

## Control Flow Instructions

### Call

Equivalent to `push PC` then `mov PC, <address>`.

- `call <reg>`
    - Encoding: `fe`
    - Latency: 26 cycles
- `call <imm16>`
    - Encoding: `fd`
    - Latency: 28 cycles

### Ret

Equivalent to `pop PC`.

- `ret`
    - Encoding: `c3`
    - Latency: 24 cycles

## Miscellaneous Instructions

### Nop

Does nothing for as many cycles as the length of the "nop chain" starting from this instruction. For example, the bytecode `6e 6f 6f 6f 6f 70` (ASCII `noooop`) sleeps for 6 cycles.

- `nop`
    - Encoding: `6e`
    - Latency: 1 cycle

### Op

If preceded by `nop` followed by zero or more `op`, does nothing. Otherwise it is undefined behavior.

- `op`
    - Encoding: `6f`
    - Latency: 1 cycle

### P

If preceded by `nop` followed by zero or more `op`, does nothing. Otherwise it is undefined behavior.

- `p`
    - Encoding: `70`
    - Latency: 1 cycle

### Syscall

Asks the kernel for something. The system call number is the lower 8 bits of the `AX` register and the arguments are placed in `R0`, `R1`, `R2`, `R3`, `R4`, and `R5`, in that order. After the system call finishes, its return value is placed in `AX`.

- `syscall`
    - Encoding: `0f`
    - Latency: 100 cycles

### Reserved

Indicates that an opcode is reserved for a future instruction set update. Any opcode that is not used by an instruction is reserved. Calling a reserved opcode is undefined behavior.
