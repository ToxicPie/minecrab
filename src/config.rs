use crate::game::map::Location;
use crate::vm::emulator;

use serde::{Deserialize, Serialize};
use serde_json;
use std::collections::HashMap;
use std::fs::File;
use std::io::Read;
use std::path::{Path, PathBuf};

#[derive(Debug)]
pub struct ConfigIoError {
    pub message: String,
}

impl<T> From<T> for ConfigIoError
where
    T: std::error::Error,
{
    fn from(value: T) -> Self {
        Self {
            message: value.to_string(),
        }
    }
}

#[derive(Serialize, Deserialize)]
pub struct GameConfiguration {
    pub user_configs: Vec<RawUserConfiguration>,
    pub default_nice: u16,
    pub initd_lifetime: u32,
    pub max_processes: usize,
    pub mapdata_path: PathBuf,
    pub crypto_spawn: HashMap<String, Vec<(i64, f64)>>,
}

#[derive(Serialize, Deserialize)]
pub struct RawUserConfiguration {
    pub initd_bytecode: PathBuf,
    pub initd_memory: PathBuf,
    pub uid: u16,
    pub spawn_point: Location,
}

pub struct UserConfiguration {
    pub initd_memory: Vec<u8>,
    pub initd_bytecode: Vec<u8>,
    pub uid: u16,
    pub spawn_point: Location,
}

pub struct KernelConfiguration {
    pub max_processes: usize,
    pub initd_lifetime: u32,
    pub default_nice: u16,
    pub crypto_spawn: HashMap<String, Vec<(i64, f64)>>,
}

impl GameConfiguration {
    pub fn load(filename: &str) -> Result<GameConfiguration, ConfigIoError> {
        Ok(serde_json::from_reader(File::open(filename)?)?)
    }

    pub fn dump(&self, filename: &str) -> Result<(), ConfigIoError> {
        Ok(serde_json::to_writer(File::create(filename)?, self)?)
    }

    pub fn get_kernel_config(&self) -> KernelConfiguration {
        KernelConfiguration {
            max_processes: self.max_processes,
            initd_lifetime: self.initd_lifetime,
            default_nice: self.default_nice,
            crypto_spawn: self.crypto_spawn.clone(),
        }
    }

    pub fn read_mapdata(&self) -> Result<Vec<u8>, ConfigIoError> {
        let mut file = File::open(&self.mapdata_path)?;
        let mut bytes = vec![];
        file.read_to_end(&mut bytes)?;
        Ok(bytes)
    }

    fn read_bytes_from_file(&self, path: &Path, length: usize) -> Result<Vec<u8>, ConfigIoError> {
        let mut file = File::open(path)?;
        let mut bytes = vec![];
        file.read_to_end(&mut bytes)?;
        if bytes.len() != length {
            Err(ConfigIoError {
                message: format!("Incorrect file size {} for file {:?}", bytes.len(), path),
            })
        } else {
            Ok(bytes)
        }
    }

    pub fn get_user_configs(&self) -> Result<Vec<UserConfiguration>, ConfigIoError> {
        self.user_configs
            .iter()
            .map(|user_config| {
                let initd_memory =
                    self.read_bytes_from_file(&user_config.initd_memory, emulator::MEMORY_SIZE)?;
                let initd_bytecode = self
                    .read_bytes_from_file(&user_config.initd_bytecode, emulator::BYTECODE_SIZE)?;
                Ok(UserConfiguration {
                    initd_memory,
                    initd_bytecode,
                    uid: user_config.uid,
                    spawn_point: user_config.spawn_point,
                })
            })
            .collect::<Result<_, _>>()
    }
}
