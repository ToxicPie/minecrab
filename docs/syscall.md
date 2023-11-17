# Minecrab Kernel System Calls

## Overview

When a process calls the `syscall` instruction (opcode `0x0f`), it informs the kernel that it wants to access some resources. Execution is then switched to the kernel temporarily, and after the system call returns, the process resumes execution.

The lower 8 bits of the AX register stores the system call number, which decides which system call is executed. System calls have 0 to 6 arguments, which are placed in the R0, R1, R2, R3, R4, and R5 registers. When a system call finishes, its return value is placed in the AX register. When a system call fails, it returns 0.

Some system calls cost cryptocurrency to execute. If the calling process's user can't afford the cost, it is not executed. When a system call fails, no cryptocurrency is charged.

## System Call Table

|Number|Name|Arg 0|Arg 1|Arg 2|Arg 3|Arg 4|Arg 5|
|------|----|-----|-----|-----|-----|-----|-----|
|0x00|GetPid|||||||
|0x01|GetUidOf|pid||||||
|0x02|Fork|||||||
|0x03|Kill|pid||||||
|0x04|GetProcessInfo|pid|addr|||||
|0x05|Detach|||||||
|0x06|Renice|||||||
|0x10|Move|x|y|||||
|0x11|ReadMap|addr|x1|y1|x2|y2||
|0x12|ReadMapDetail|addr|x1|y1|x2|y2||
|0x13|FetchChallenge|addr|max_len|||||
|0x14|SolveChallenge|nonce[0]|nonce[1]|nonce[2]|nonce[3]|||
|0x15|PathFind|addr|x|y|n|||
|0x20|Attack1|x|y|||||
|0x21|Attack2|x|y|||||
|0x30|UpdateCode|mem_addr|code_addr|n||||
|0x31|ShareMemory|pid|dst_addr|src_addr|n|||
|0x40|<ruby>Ｅｘｐｌｏｓｉｏｎ<rt>エクスプロージョン</rt></ruby>|x|y|||||
|0x41|Teleport|pid||||||

## Game Syscalls

### Move

`move(x, y)`

> Cost: 1 Dogecoin

Moves the process to the coordinates `(x, y)`. `x` and `y` are taken modulo 256. The destination must be within a 3x3 square centered on the process, and be different from the current location. Init cannot move.

The destination cannot be a wall or contain another process. Walls do not block a move, for example in the following configuration
```
....
##B.
.A##
....
```
where `#` denotes a wall, a process is allowed to move from A to B.

Return value:
- On success: 1
- On failure: 0

### ReadMap

`read_map(addr, x1, y1, x2, y2)`

> Cost: 1 Dogecoin for every 256 cells read (rounded up)

For every `x` in the range `[x1, x2]` and for every `y` in the range `[y1, y2]`, wrapping around at map borders, write a byte describing the status of the cell at `(x, y)`:
- 0 means the cell's type is land.
- 1 means the cell's type is wall.

A total of `((x2 - x1) mod 256 + 1) * ((y2 - y1) mod 256 + 1)` bytes are written to the memory region starting at `addr`. Bytes are written in x-major order, for example `map[1][1], map[1][2], map[1][3], ..., map[2][1], map[2][2], ...`.

Return value:
- On success: The number of bytes written modulo 2^16
- On failure: 0

### ReadMapDetail

`read_map_detail(addr, x1, y1, x2, y2)`

> Cost: 1 Dogecoin for every 64 cells read (rounded up)

For every `x` in the range `[x1, x2]` and for every `y` in the range `[y1, y2]`, wrapping around at map borders, write 3 bytes describing the status of the cell at `(x, y)`:
- If the first byte is 0, it means the cell is empty land. Then 2 zero bytes follow.
- If the first byte is 1, it means the cell contains a wall. Then 2 zero bytes follow.
- If the first byte is 2, it means the cell contains a process. The next 2 bytes denote the process's pid in little-endian.
- If the first byte is 3, it means the cell contains a crypto challenge. The next 2 bytes denote the challenge's numeric id in little-endian.

A total of `3 * ((x2 - x1) mod 256 + 1) * ((y2 - y1) mod 256 + 1)` bytes are written to the memory region starting at `addr`. Bytes are written in x-major order, for example `map[1][1], map[1][2], map[1][3], ..., map[2][1], map[2][2], ...`. At most 65535 bytes can be written.

Init has special privileges and can read the map at arbitrary locations. For non-init processes, random bytes will be returned when reading a cell `(x, y)` unless it's within a 9x9 square centered on the process.

Return value:
- On success: The number of bytes written modulo 2^16
- On failure: 0

### FetchChallenge

`fetch_challenge(addr, max_len)`

> Cost: Free

Writes info about the crypto challenge at the calling process's location into the memory area starting from `addr`. The target process must be owned by the same user. The structure of the data written is as follows. Data are in little-endian. If total data length exceeds `max_len`, nothing is written.

|Offset|Length|Data|
|:----:|:----:|----|
|0x00  |0x02  |Numeric ID|
|0x02  |0x02  |Difficulty modulo 2^32|
|0x04  |0x02  |Challenge data length|
|0x06  |?     |Challenge data|

Return value:
- On success: The number of bytes written
- On failure: 0

### SolveChallenge

`solve_challenge(nonce[0], nonce[1], nonce[2], nonce[3])`

> Cost: Free

Attempts to solve the crypto challenge at the calling process's location. If the attempt succeeds, the user is awarded the challenge's reward. If the attempt fails or there is no crypto at the location, the calling process is killed.

Return value:
- Correct: 1
- Incorrect or failure: 💀

### Attack1

`attack1(x, y)`

> Cost: 8 DogeCoin

Attacks the process at the location `(x, y)`. It's possible to attack processes of the same user or even the calling process itself. The target location must be within a 9x9 square centered on the calling process.

On a successful attack, the target process loses 1 lifetime.

Return value:
- On success: 1
- On failure: 0

### Attack2

`attack2(x, y)`

> Cost: 16 DogeCoin

Attacks the process at the location `(x, y)`. It's possible to attack processes of the same user or even the calling process itself. The target location must be within a 5x5 square centered on the calling process.

On a successful attack, the target process executes undefined behavior.

Return value:
- On success: 1
- On failure: 0

### PathFind

`path_find(addr, x, y, n)`

> Cost: `n` DogeCoins

Finds a shortest path from the calling process's location to the target location `(x, y)` in at most `n` moves. `n` is at most 16.

On success, `2 * len(path)` bytes are written to memory starting at `addr` denoting coordinates on the found path: `x1, y1, x2, y2, x3, y3, ..., x, y`, i.e., the path is `current location -> (x1, y1) -> (x2, y2) -> ... -> (x, y)`. All tiles on the path will contain no other processes at the time of calling.

Return value:
- On success: Length of the path
- On failure: 0

### <ruby>Ｅｘｐｌｏｓｉｏｎ<rt>エクスプロージョン</rt></ruby>

`explosion(x, y)`

> Cost: -10000 StarSleepShortage

Attacks all processes within a 15x15 square centered on the target location `(x, y)`. Init cannot use this attack.

On a successful attack, all non-init processes in the target area are killed.

This system call can only be used once per user per game.

Return value:
- On success: The number of targets attacked
- On failure: 0

### Teleport

`teleport(pid)`

> Cost: 32 DogeCoin

Instantly moves to a random empty square within a 5x5 square centered on the target process. The target process must be owned by the same user as the calling process. Init cannot use this system call. 5 attempts are made to choose an empty square before this system call fails.

Return value:
- On success: 1
- On failure: 0

## Process Syscalls

### GetPid

`get_pid()`

> Cost: Free

Gets the PID of the calling process.

Return value:
- The calling process's PID

### GetUidOf

`get_uid(pid)`

> Cost: Free

Gets the UID of a process's owner. The target process must be alive.

Return value:
- On success: The UID
- On failure: 0

### Fork

`fork()`

> Cost: 4 Ethereum

Creates a new child process that has the same bytecode, memory and registers as the original. The calling process is the parent and the cloned process is the child. The newly created child process has nice 0 and is not executed until the next tick.

The new process will have half the remaining lifetime as the parent process, rounded down. If the parent process is not init, it will lose the same amount of lifetime that was given to the child. If the calling process has less than 2 ticks of remaining lifetime, `fork` fails.

Up to 5 attempts are made to spawn the process on a random square within a 5x5 area centered on the parent process. An attempt fails if the chosen square already contains a process or challenge. If all attempts fail, `fork` fails.

Return value:
- On success: the child's PID to the parent process, -1 (0xffff) to the child process
- On failure: 0

### Kill

`kill(pid)`

> Cost: 2 Ethereum

Kills a process. The target process must be the calling process itself or its descendant and must be alive. The target process and all its descendant processes are terminated and removed from the game immediately.

Return value:
- On success: 1
- On failure: 0

### GetProcessInfo

`get_process_info(pid, addr)`

> Cost: Free

Writes info about a process into the memory area starting from `addr`. The target process must be owned by the same user. 10 bytes are written, and the structure of the data written is as follows. Data are in little-endian.

|Offset|Length|Data|
|:----:|:----:|----|
|0x00  |0x01  |Location x|
|0x01  |0x01  |Location y|
|0x02  |0x04  |Remaining tick count|
|0x06  |0x02  |Nice value|
|0x08  |0x02  |Parent PID (0 if the process is init)|

Return value:
- On success: 1
- On failure: 0

### Detach

`detach()`

> Cost: 1 Ethereum

Detaches the calling process from its parent process. The parent process is changed to init. Init cannot be detached.

Return value:
- On success: 1
- On failure: 0

### Renice

`renice()`

> Cost: 10 Ethereum

Increases the nice value of the calling process by 1. The new nice value will take effect from the next tick.

Return value:
- On success: 1
- On failure: 0

## Misc Syscalls

### UpdateCode

`update_code(mem_addr, code_addr, n)`

> Cost: 1 Ethereum for every 1024 bytes copied (rounded up)

Modifies the bytecode of the calling process by copying `n + 1` bytes from memory starting from `mem_addr` into its bytecode starting from `code_addr`.

Note that `n` is the number of bytes to copy minus 1.

Return value:
- On success: 1
- On failure: 0

### ShareMemory

`share_memory(pid, dst_addr, src_addr, n)`

> Cost: 1 Ethereum for every 1024 bytes copied (rounded up)

Copies memory from a the calling process to another process. `n + 1` bytes starting from `src_addr` are copied to the target process's memory starting from `dst_addr`. The target process must be a descendant of the calling process.

Note that `n` is the number of bytes to copy minus 1.

Return value:
- On success: 1
- On failure: 0

### Reserved

Indicates that a syscall number is reserved. Any syscall number that is not used by a system call is reserved. Calling a reserved system call is undefined behavior.
