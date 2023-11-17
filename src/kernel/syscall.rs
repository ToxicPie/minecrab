use rand::random;

use crate::game::crypto::{wallet, Wallet};
use crate::game::map::chebyshev_distance;
use crate::game::replay::{log_event, GameEvent};
use crate::kernel::Kernel;

pub type SyscallArgs = (u16, u16, u16, u16, u16, u16);

pub trait Syscall {
    fn get_number(&self) -> u8;
    fn compute_cost(&self, args: SyscallArgs) -> Wallet;
    fn call(&self, kernel: &mut Kernel, pid: u16, args: SyscallArgs) -> Option<u16>;
}

pub struct SyscallTable {
    _used: [bool; Self::TABLE_SIZE],
    table: [&'static dyn Syscall; Self::TABLE_SIZE],
}

impl SyscallTable {
    const TABLE_SIZE: usize = 256;
    pub const fn new() -> Self {
        Self {
            _used: [false; Self::TABLE_SIZE],
            table: [&Reserved {}; Self::TABLE_SIZE],
        }
    }
    pub fn get_syscall(&self, number: u8) -> &'static dyn Syscall {
        self.table[number as usize]
    }
}

macro_rules! syscall_category {
    ($category_fn:ident() => [
        $($name:ident<$number:literal> {
            compute_cost($($cost_args:ident),*) $cost:tt
            call($kernel:ident, $pid:ident $(,$call_args:ident)*) $call:tt
        }),* $(,)?
    ]) => {
        $(
            struct $name {}
            impl Syscall for $name {
                fn get_number(&self) -> u8 {
                    $number
                }
                fn compute_cost(&self, ($($cost_args,)* ..): SyscallArgs) -> Wallet {
                    $cost
                }
                fn call(
                    &self,
                    $kernel: &mut Kernel,
                    $pid: u16,
                    ($($call_args,)* ..): SyscallArgs
                ) -> Option<u16> {
                    $call
                }
            }
        )*
        impl SyscallTable {
            pub const fn $category_fn(mut self) -> Self {
                $(
                    if self._used[$number as usize] {
                        panic!(concat!("duplicate syscall number ", stringify!($number)));
                    }
                    self._used[$number as usize] = true;
                    self.table[$number as usize] = &$name {};
                )*
                self
            }
        }
    };
}

pub const SYSCALL_TABLE: SyscallTable = SyscallTable::new()
    .make_process_syscalls()
    .make_game_syscalls()
    .make_misc_syscalls();

syscall_category! {
    make_process_syscalls() => [
        GetPid<0x00> {
            compute_cost() {
                wallet!()
            }
            call(_kernel, pid) {
                Some(pid)
            }
        },

        GetUidOf<0x01> {
            compute_cost() {
                wallet!()
            }
            call(kernel, _pid, target_pid) {
                if !kernel.has_process(target_pid) {
                    return None;
                }
                Some(kernel.get_process_owner(target_pid))
            }
        },

        Fork<0x02> {
            compute_cost() {
                wallet!(Ethereum: 4)
            }
            call(kernel, pid) {
                kernel.fork_process(pid)
            }
        },

        Kill<0x03> {
            compute_cost() {
                wallet!(Ethereum: 2)
            }
            call(kernel, pid, target_pid) {
                if !kernel.has_process(target_pid)
                    || !kernel.is_self_or_descendent_process(pid, target_pid)
                    || kernel.get_process(target_pid).is_init()
                {
                    return None;
                }
                kernel.kill_process_recursive(target_pid);
                Some(1)
            }
        },

        GetProcessInfo<0x04> {
            compute_cost() {
                wallet!(StarSleepShortage: -1)
            }
            call(kernel, pid, target_pid, addr) {
                if !kernel.has_process(target_pid)
                    || kernel.get_process_owner(pid) != kernel.get_process_owner(target_pid)
                {
                    return None;
                }
                let (x, y) = kernel.get_process_location(target_pid);
                let process = kernel.get_process(target_pid);
                let lifetime = process.lifetime;
                let nice = process.nice;
                let ppid = process.ppid.unwrap_or(0);
                let data = [
                    &x.to_le_bytes()[..],
                    &y.to_le_bytes()[..],
                    &lifetime.to_le_bytes()[..],
                    &nice.to_le_bytes()[..],
                    &ppid.to_le_bytes()[..],
                ].concat();
                kernel.get_process_mut(pid).emulator.write_bytes_to_mem(addr, &data);
                Some(1)
            }
        },

        Detach<0x05> {
            compute_cost() {
                wallet!(Ethereum: 1)
            }
            call(kernel, pid) {
                if kernel.get_process(pid).is_init() {
                    return None;
                }
                log_event(GameEvent::Detach { pid });
                let init_pid = kernel.get_owner_user(pid).initd_pid.unwrap();
                kernel.remove_from_parent(pid);
                kernel.get_process_mut(init_pid).children.push(pid);
                kernel.get_process_mut(pid).ppid = Some(init_pid);
                Some(1)
            }
        },

        Renice<0x06> {
            compute_cost() {
                wallet!(Ethereum: 10)
            }
            call(kernel, pid) {
                kernel.get_process_mut(pid).renice();
                Some(1)
            }
        },
    ]
}

syscall_category! {
    make_game_syscalls() => [
        Move<0x10> {
            compute_cost() {
                wallet!(
                    DogeCoin: 1,
                    StarSleepShortage: -1
                )
            }
            call(kernel, pid, x, y) {
                if kernel.get_process(pid).is_init() {
                    return None;
                }
                let location = (x as u8, y as u8);
                if kernel.move_process_to(pid, location) {
                    Some(1)
                } else {
                    None
                }
            }
        },

        ReadMap<0x11> {
            compute_cost(_addr, x1, y1, x2, y2) {
                let x1 = x1 as u8;
                let y1 = y1 as u8;
                let x2 = x2 as u8;
                let y2 = y2 as u8;
                let dx = x2.wrapping_sub(x1) as i64 + 1;
                let dy = y2.wrapping_sub(y1) as i64 + 1;
                wallet!(DogeCoin: (dx * dy).div_ceil(256))
            }
            call(kernel, pid, addr, x1, y1, x2, y2) {
                let x1 = x1 as u8;
                let y1 = y1 as u8;
                let x2 = x2 as u8;
                let y2 = y2 as u8;
                let dx = x2.wrapping_sub(x1);
                let dy = y2.wrapping_sub(y1);
                let mut data = vec![];
                for i in 0..=dx {
                    for j in 0..=dy {
                        let x = x1.wrapping_add(i);
                        let y = y1.wrapping_add(j);
                        let cell = kernel.get_map_cell((x, y));
                        data.push(cell.status());
                    }
                }
                kernel.get_process_mut(pid).emulator.write_bytes_to_mem(addr, &data);
                Some(data.len() as u16)
            }
        },

        ReadMapDetail<0x12> {
            compute_cost(_addr, x1, y1, x2, y2) {
                let x1 = x1 as u8;
                let y1 = y1 as u8;
                let x2 = x2 as u8;
                let y2 = y2 as u8;
                let dx = x2.wrapping_sub(x1) as i64 + 1;
                let dy = y2.wrapping_sub(y1) as i64 + 1;
                wallet!(DogeCoin: (dx * dy).div_ceil(64))
            }
            call(kernel, pid, addr, x1, y1, x2, y2) {
                let x1 = x1 as u8;
                let y1 = y1 as u8;
                let x2 = x2 as u8;
                let y2 = y2 as u8;
                let dx = x2.wrapping_sub(x1);
                let dy = y2.wrapping_sub(y1);
                if (dx as i64 + 1) * (dy as i64 + 1) * 3 > 65535 {
                    return None;
                }
                let location = kernel.get_process_location(pid);
                let is_init = kernel.get_process(pid).is_init();
                let mut data = vec![];
                for i in 0..=dx {
                    for j in 0..=dy {
                        let x = x1.wrapping_add(i);
                        let y = y1.wrapping_add(j);
                        if !is_init && chebyshev_distance(location, (x, y)) <= 4 {
                            let cell = kernel.get_map_cell((x, y));
                            data.extend(cell.status_detail());
                        } else {
                            data.extend(random::<[u8; 3]>());
                        }
                    }
                }
                kernel.get_process_mut(pid).emulator.write_bytes_to_mem(addr, &data);
                Some(data.len() as u16)
            }
        },

        FetchChallenge<0x13> {
            compute_cost() {
                wallet!(StarSleepShortage: -1)
            }
            call(kernel, pid, addr, max_len) {
                let data = kernel.fetch_challenge_data(pid)?;
                if data.len() > max_len as usize {
                    return None;
                }
                kernel.get_process_mut(pid).emulator.write_bytes_to_mem(addr, &data);
                Some(data.len() as u16)
            }
        },

        SolveChallenge<0x14> {
            compute_cost() {
                wallet!(DogeCoin: 1)
            }
            call(kernel, pid, nonce0, nonce1, nonce2, nonce3) {
                let nonce = (nonce0, nonce1, nonce2, nonce3);
                Some(kernel.solve_challenge(pid, nonce))
            }
        },

        Attack1<0x20> {
            compute_cost() {
                wallet!(DogeCoin: 8)
            }
            call(kernel, pid, x, y) {
                let attacker_location = kernel.get_process_location(pid);
                let target_location = (x as u8, y as u8);
                if chebyshev_distance(attacker_location, target_location) > 4 {
                    return None;
                }
                let Some(target_pid) = kernel.get_map_cell(target_location).get_process() else {
                    return None;
                };
                let target_process = kernel.get_process_mut(target_pid);
                target_process.lifetime = target_process.lifetime.saturating_sub(1);
                log_event(GameEvent::Attack {
                    attacker_pid:pid,
                    defender_pid: target_pid,
                });
                Some(1)
            }
        },

        Attack2<0x21> {
            compute_cost() {
                wallet!(DogeCoin: 16)
            }
            call(kernel, pid, x, y) {
                let attacker_location = kernel.get_process_location(pid);
                let target_location = (x as u8, y as u8);
                if chebyshev_distance(attacker_location, target_location) > 2 {
                    return None;
                }
                let Some(target_pid) = kernel.get_map_cell(target_location).get_process() else {
                    return None;
                };
                let target_process = kernel.get_process_mut(target_pid);
                target_process.emulator.nasal_demons();
                log_event(GameEvent::Attack {
                    attacker_pid: pid,
                    defender_pid: target_pid,
                });
                Some(1)
            }
        },
    ]
}

syscall_category! {
    make_misc_syscalls() => [
        UpdateCode<0x30> {
            compute_cost(_mem_addr, _code_addr, n) {
                wallet!(Ethereum: n.div_ceil(1024) as _)
            }
            call(kernel, pid, mem_addr, code_addr, n) {
                let emulator = &mut kernel.get_process_mut(pid).emulator;
                let count = n as usize + 1;
                let code = emulator.peek_bytes_from_mem(mem_addr, count);
                emulator.write_bytes_to_code(code_addr, &code);
                Some(1)
            }
        },

        ShareMemory<0x31> {
            compute_cost(_target_pid, _dst_addr, _src_addr, n) {
                wallet!(Ethereum: n.div_ceil(1024) as _)
            }
            call(kernel, pid, target_pid, dst_addr, src_addr, n) {
                if pid == target_pid
                    || !kernel.has_process(target_pid)
                    || !kernel.is_self_or_descendent_process(pid, target_pid)
                {
                    return None;
                }
                let self_emulator = &mut kernel.get_process_mut(pid).emulator;
                let count = n as usize + 1;
                let bytes = self_emulator.peek_bytes_from_mem(src_addr, count);
                let target_emulator = &mut kernel.get_process_mut(target_pid).emulator;
                target_emulator.write_bytes_to_mem(dst_addr, &bytes);
                Some(1)
            }
        },

        Reserved<0xff> {
            compute_cost() {
                wallet!(StarSleepShortage: -10)
            }
            call(kernel, pid) {
                kernel.get_process_mut(pid).emulator.nasal_demons();
                None
            }
        },
    ]
}
