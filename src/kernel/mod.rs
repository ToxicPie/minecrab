use crate::config::{KernelConfiguration, UserConfiguration};
use crate::game::crypto::Wallet;
use crate::game::map::{chebyshev_distance, GameMap, Location, MapCell};
use crate::game::replay::{log_event, GameEvent};
use crate::kernel::process::Process;
use crate::kernel::user::User;
use crate::vm::emulator::Emulator;

use rand::Rng;
use std::collections::HashMap;

pub mod process;
pub mod syscall;
pub mod user;

pub struct Kernel {
    config: KernelConfiguration,
    process_table: HashMap<u16, Process>,
    user_table: HashMap<u16, User>,
    game_map: GameMap,
}

impl Kernel {
    pub fn new(kernel_config: KernelConfiguration, game_map: GameMap) -> Self {
        Kernel {
            process_table: HashMap::new(),
            user_table: HashMap::new(),
            game_map,
            config: kernel_config,
        }
    }

    pub fn setup_users(&mut self, user_configs: Vec<UserConfiguration>) {
        for UserConfiguration {
            initd_memory,
            initd_bytecode,
            uid,
            spawn_point,
        } in user_configs
        {
            let pid = self.allocate_pid();
            let initd_process = Process {
                pid,
                ppid: None,
                children: vec![],
                uid,
                lifetime: self.config.initd_lifetime,
                nice: self.config.default_nice,
                emulator: Emulator::new(initd_memory, initd_bytecode),
            };
            self.process_table.insert(pid, initd_process);
            self.game_map.add_process_to_map(pid, spawn_point);
            log_event(GameEvent::NewProcess {
                uid,
                ppid: None,
                pid,
                location: (spawn_point.0, spawn_point.1),
            });
            let user = User {
                uid,
                initd_pid: Some(pid),
                score: 0,
                num_processes: 1,
                wallet: Wallet::get_newbie_welcome_pack(),
            };
            self.user_table.insert(uid, user);
        }
    }

    fn run_process_tick(&mut self, pid: u16) {
        let Some(process) = self.process_table.get_mut(&pid) else {
            return;
        };
        process.lifetime = process.lifetime.saturating_sub(1);
        process.emulator.increment_ts();
        let cycle_count = process.get_execution_limit();
        let user = self.get_owner_user_mut(pid);
        let mut cycle_count = user.compute_sleep_debt(cycle_count);
        user.score += cycle_count as i64 / 100;
        log_event(GameEvent::ScoreUpdate {
            uid: user.uid,
            new_score: user.score,
        });
        loop {
            let Some(process) = self.process_table.get_mut(&pid) else {
                return;
            };
            match process.emulator.run_until_interrupt(&mut cycle_count) {
                Some(syscall) => {
                    let args = process.emulator.get_syscall_args();
                    let cost = syscall.compute_cost(args);
                    let uid = process.uid;
                    if !self.get_user(uid).wallet.can_afford(&cost) {
                        self.get_process_mut(pid)
                            .emulator
                            .set_syscall_return_value(0);
                        continue;
                    }
                    match syscall.call(self, pid, args) {
                        Some(ret) => {
                            self.get_user_mut(uid).wallet -= &cost;
                            log_event(GameEvent::WalletUpdate {
                                uid,
                                new_wallet: &self.get_user(uid).wallet,
                            });
                            let Some(process) = self.process_table.get_mut(&pid) else {
                                return;
                            };
                            process.emulator.set_syscall_return_value(ret);
                        }
                        None => {
                            self.get_process_mut(pid)
                                .emulator
                                .set_syscall_return_value(0);
                        }
                    }
                }
                None => return,
            }
        }
    }

    pub fn tick_processes(&mut self) {
        let mut process_queue = self.process_table.keys().copied().collect::<Vec<_>>();
        process_queue.sort_by(|pid1, pid2| {
            let process1 = self.get_process(*pid1);
            let process2 = self.get_process(*pid2);
            process1
                .nice
                .cmp(&process2.nice)
                .reverse()
                .then(pid1.cmp(pid2))
        });
        for pid in process_queue.iter() {
            self.run_process_tick(*pid);
        }
        for pid in process_queue {
            if let Some(process) = self.process_table.get(&pid) {
                if process.lifetime == 0 {
                    self.kill_process_recursive(pid);
                }
            }
        }
    }

    pub fn run_full_game(&mut self) {
        while !self.process_table.is_empty() {
            self.game_map.tick(&self.config.crypto_spawn);
            self.tick_processes();
        }
        for user in self.user_table.values_mut() {
            user.convert_wallet_to_score();
        }
    }

    pub fn get_user(&self, uid: u16) -> &User {
        self.user_table.get(&uid).unwrap()
    }

    pub fn get_user_mut(&mut self, uid: u16) -> &mut User {
        self.user_table.get_mut(&uid).unwrap()
    }

    pub fn has_process(&self, pid: u16) -> bool {
        self.process_table.contains_key(&pid)
    }

    pub fn get_process(&self, pid: u16) -> &Process {
        self.process_table.get(&pid).unwrap()
    }

    pub fn get_process_mut(&mut self, pid: u16) -> &mut Process {
        self.process_table.get_mut(&pid).unwrap()
    }

    pub fn get_process_owner(&self, pid: u16) -> u16 {
        self.get_process(pid).uid
    }

    pub fn get_owner_user(&self, pid: u16) -> &User {
        self.get_user(self.get_process_owner(pid))
    }

    pub fn get_owner_user_mut(&mut self, pid: u16) -> &mut User {
        self.get_user_mut(self.get_process_owner(pid))
    }

    pub fn get_process_location(&self, pid: u16) -> Location {
        self.game_map.get_process_location(pid)
    }

    pub fn get_map_cell(&self, location: Location) -> &MapCell {
        self.game_map.get_cell(location)
    }

    pub fn remove_from_parent(&mut self, pid: u16) {
        let Some(ppid) = self.get_process(pid).ppid else {
            return;
        };
        let parent = self.get_process_mut(ppid);
        parent.children.retain(|child| *child != pid);
    }

    fn kill_process(&mut self, pid: u16) {
        log_event(GameEvent::Kill { pid });
        self.get_owner_user_mut(pid).num_processes -= 1;
        if self.get_process(pid).is_init() {
            self.get_owner_user_mut(pid).initd_pid = None;
        }
        self.game_map.remove_process_from_map(pid);
        self.remove_from_parent(pid);
        self.process_table.remove(&pid);
    }

    pub fn kill_process_recursive(&mut self, pid: u16) {
        let chilren = self.get_process(pid).children.clone();
        for child in chilren {
            self.kill_process_recursive(child);
        }
        self.kill_process(pid);
    }

    pub fn is_self_or_descendent_process(&self, self_pid: u16, target_pid: u16) -> bool {
        if self_pid == target_pid {
            return true;
        }
        self.get_process(self_pid)
            .children
            .iter()
            .any(|child_pid| self.is_self_or_descendent_process(*child_pid, target_pid))
    }

    pub fn move_process_to(&mut self, pid: u16, location: Location) -> bool {
        let old_location = self.game_map.get_process_location(pid);
        if chebyshev_distance(old_location, location) != 1 {
            return false;
        }
        self.game_map.move_process_to(pid, location)
    }

    pub fn teleport_process_to(&mut self, pid: u16, location: Location) -> bool {
        let Some(new_location) = self.game_map.find_empty_location_nearby(location, 5, 2) else {
            return false;
        };
        self.game_map.move_process_to(pid, new_location)
    }

    pub fn pathfind_process_to(
        &mut self,
        pid: u16,
        location: Location,
        max_len: usize,
    ) -> Option<Vec<Location>> {
        let old_location = self.game_map.get_process_location(pid);
        self.game_map.pathfind(old_location, location, max_len)
    }

    pub fn fetch_challenge_data(&self, pid: u16) -> Option<Vec<u8>> {
        let location = self.game_map.get_process_location(pid);
        self.game_map.get_cell(location).crypto_data()
    }

    pub fn solve_challenge(&mut self, pid: u16, nonce: (u16, u16, u16, u16)) -> u16 {
        let location = self.game_map.get_process_location(pid);
        if let Some(wallet) = self.game_map.get_cell_mut(location).solve_crypto(nonce) {
            let user = self.get_owner_user_mut(pid);
            user.wallet += &wallet;
            log_event(GameEvent::ChallengeSolved { pid, location });
            log_event(GameEvent::WalletUpdate {
                uid: user.uid,
                new_wallet: &user.wallet,
            });
            1
        } else {
            self.kill_process_recursive(pid);
            0
        }
    }

    fn allocate_pid(&self) -> u16 {
        let mut rng = rand::thread_rng();
        loop {
            let random_pid = rng.gen_range(1..=65534);
            if !self.process_table.contains_key(&random_pid) {
                break random_pid;
            }
        }
    }

    pub fn fork_process(&mut self, pid: u16) -> Option<u16> {
        if self.get_owner_user(pid).num_processes >= self.config.max_processes
            || self.get_process(pid).lifetime < 2
        {
            return None;
        }
        let child_location = self.game_map.find_empty_location_nearby(
            self.game_map.get_process_location(pid),
            5,
            2,
        )?;
        let child_pid = self.allocate_pid();
        self.game_map.add_process_to_map(child_pid, child_location);
        let parent_process = self.get_process_mut(pid);
        let half_lifetime = parent_process.lifetime / 2;
        if !parent_process.is_init() {
            parent_process.lifetime -= half_lifetime;
        }
        parent_process.children.push(child_pid);
        let mut child_process = Process {
            pid: child_pid,
            ppid: Some(pid),
            children: vec![],
            uid: parent_process.uid,
            lifetime: half_lifetime,
            nice: 0,
            emulator: parent_process.emulator.clone(),
        };
        log_event(GameEvent::NewProcess {
            uid: parent_process.uid,
            ppid: Some(pid),
            pid: child_pid,
            location: (child_location.0, child_location.1),
        });
        child_process.emulator.set_syscall_return_value(0xffff);
        self.process_table.insert(child_pid, child_process);
        self.get_user_mut(self.get_process_owner(pid)).num_processes += 1;
        Some(child_pid)
    }
}
