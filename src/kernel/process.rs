use crate::game::replay::{log_event, GameEvent};
use crate::vm::emulator;

pub struct Process {
    pub pid: u16,
    pub ppid: Option<u16>,
    pub children: Vec<u16>,
    pub uid: u16,
    pub lifetime: u32,
    pub nice: u16,
    pub emulator: emulator::Emulator,
}

impl Process {
    pub fn is_init(&self) -> bool {
        self.ppid.is_none()
    }

    pub fn renice(&mut self) {
        self.nice = self.nice.saturating_add(1);
        log_event(GameEvent::Renice {
            pid: self.pid,
            new_nice: self.nice,
        });
    }

    pub fn get_execution_limit(&self) -> usize {
        match self.nice {
            0 => 1000,
            1..=5 => self.nice as usize * 200 + 1000,
            6..=10 => self.nice as usize * 150 + 1250,
            11..=15 => self.nice as usize * 100 + 1750,
            16..=20 => self.nice as usize * 50 + 2500,
            21.. => 3500,
        }
    }
}
