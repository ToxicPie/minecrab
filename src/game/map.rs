use crate::game::crypto::*;
use crate::game::replay::{log_event, GameEvent};

use rand::{thread_rng, Rng};
use std::collections::HashMap;

pub const MAP_WIDTH: usize = 256;
pub const MAP_HEIGHT: usize = 256;

#[derive(PartialEq)]
pub enum CellType {
    Land,
    Wall,
}

pub type Location = (u8, u8);

pub fn chebyshev_distance((x1, y1): Location, (x2, y2): Location) -> u8 {
    let dx = std::cmp::min(x1.wrapping_sub(x2), x2.wrapping_sub(x1));
    let dy = std::cmp::min(y1.wrapping_sub(y2), y2.wrapping_sub(y1));
    std::cmp::max(dx, dy)
}

#[derive(Debug)]
pub struct GameMapError {}

pub struct MapCell {
    cell_type: CellType,
    crypto: Option<Box<dyn CryptoChallenge>>,
    process: Option<u16>,
}

impl MapCell {
    pub fn new(cell_type: CellType) -> Self {
        Self {
            cell_type,
            crypto: None,
            process: None,
        }
    }

    pub fn is_empty(&self) -> bool {
        self.cell_type == CellType::Land && self.crypto.is_none() && self.process.is_none()
    }

    pub fn get_process(&self) -> Option<u16> {
        self.process
    }

    pub fn status(&self) -> u8 {
        match self.cell_type {
            CellType::Land => 0,
            CellType::Wall => 1,
        }
    }

    pub fn status_detail(&self) -> [u8; 3] {
        if let Some(pid) = self.process {
            [2, pid as u8, (pid >> 8) as u8]
        } else if let Some(ref crypto) = self.crypto {
            let id = crypto.get_numeric_id();
            [3, id as u8, (id >> 8) as u8]
        } else {
            match self.cell_type {
                CellType::Land => [0, 0, 0],
                CellType::Wall => [1, 0, 0],
            }
        }
    }

    pub fn crypto_data(&self) -> Option<Vec<u8>> {
        let Some(ref crypto) = self.crypto else {
            return None;
        };
        let mut data = vec![];
        let challenge = crypto.get_challenge_data();
        data.extend(crypto.get_numeric_id().to_le_bytes());
        data.extend(crypto.get_difficulty().to_le_bytes());
        data.extend((challenge.len() as u16).to_le_bytes());
        data.extend(challenge);
        Some(data)
    }

    pub fn solve_crypto(&mut self, nonce: (u16, u16, u16, u16)) -> Option<Wallet> {
        let Some(ref crypto) = self.crypto else {
            return None;
        };
        if crypto.verify(nonce) {
            Some(self.crypto.take().unwrap().get_reward())
        } else {
            None
        }
    }
}

pub struct GameMap {
    map: Vec<Vec<MapCell>>,
    process_location_map: HashMap<u16, Location>,
}

impl GameMap {
    pub fn from_map_data(map_data: &[u8]) -> Result<Self, GameMapError> {
        if map_data.len() != MAP_WIDTH * MAP_HEIGHT {
            return Err(GameMapError {});
        }
        let rows = map_data.chunks_exact(MAP_WIDTH);
        let parsed_map = rows
            .map(|row| {
                row.iter()
                    .map(|cell| match *cell {
                        0 => Ok(MapCell::new(CellType::Land)),
                        1 => Ok(MapCell::new(CellType::Wall)),
                        _ => Err(GameMapError {}),
                    })
                    .collect()
            })
            .collect::<Result<_, _>>()?;
        Ok(GameMap {
            map: parsed_map,
            process_location_map: HashMap::new(),
        })
    }

    pub fn get_cell(&self, location: Location) -> &MapCell {
        &self.map[location.0 as usize][location.1 as usize]
    }

    pub fn get_cell_mut(&mut self, location: Location) -> &mut MapCell {
        &mut self.map[location.0 as usize][location.1 as usize]
    }

    pub fn get_process_at(&self, location: Location) -> Option<u16> {
        self.get_cell(location).get_process()
    }

    pub fn get_process_location(&self, pid: u16) -> Location {
        *self.process_location_map.get(&pid).unwrap()
    }

    pub fn add_process_to_map(&mut self, pid: u16, location: Location) {
        self.process_location_map.insert(pid, location);
    }

    pub fn remove_process_from_map(&mut self, pid: u16) {
        self.get_cell_mut(self.get_process_location(pid)).process = None;
        self.process_location_map.remove(&pid);
    }

    pub fn find_empty_location_nearby(
        &self,
        (x, y): Location,
        attempts: usize,
        range: i8,
    ) -> Option<Location> {
        let mut rng = rand::thread_rng();
        for _ in 0..attempts {
            let x_diff = rng.gen_range(-range..=range);
            let y_diff = rng.gen_range(-range..=range);
            let new_x = x.wrapping_add_signed(x_diff);
            let new_y = y.wrapping_add_signed(y_diff);
            let new_location = (new_x, new_y);
            if self.get_cell(new_location).is_empty() {
                return Some(new_location);
            }
        }
        None
    }

    fn try_add_crypto_at_random(&mut self, challenge: Box<dyn CryptoChallenge>) {
        let location = (rand::random(), rand::random());
        let cell = self.get_cell(location);
        if cell.is_empty() {
            log_event(GameEvent::NewChallenge {
                challenge_type: challenge.get_name(),
                difficulty: challenge.get_difficulty(),
                location,
            });
            self.get_cell_mut(location).crypto = Some(challenge);
        }
    }

    fn add_cryptos(&mut self, name: &str, distributions: &[(i64, f64)]) {
        let mut rng = thread_rng();
        for &(difficulty, probability) in distributions {
            if rng.gen::<f64>() < probability {
                if let Some(challenge) = generate_crypto_challenge(name, difficulty) {
                    self.try_add_crypto_at_random(challenge);
                }
            }
        }
    }

    pub fn tick(&mut self, config: &HashMap<String, Vec<(i64, f64)>>) {
        for (name, distributions) in config.iter() {
            self.add_cryptos(name, distributions);
        }
    }

    pub fn move_process_to(&mut self, pid: u16, location: Location) -> bool {
        if self.get_cell(location).process.is_some() {
            return false;
        }
        self.get_cell_mut(self.get_process_location(pid)).process = None;
        *self.process_location_map.get_mut(&pid).unwrap() = location;
        self.get_cell_mut(location).process = Some(pid);
        log_event(GameEvent::Move { pid, location });
        true
    }
}
