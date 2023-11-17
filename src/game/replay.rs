use crate::game::crypto::Wallet;
use crate::game::map::Location;

use serde::Serialize;

#[derive(Serialize)]
#[serde(tag = "type")]
pub enum GameEvent<'a> {
    InitMap {
        map_data: &'a [u8],
        map_height: usize,
        map_width: usize,
    },
    Move {
        pid: u16,
        location: Location,
    },
    Attack {
        attacker_pid: u16,
        defender_pid: u16,
    },

    NewProcess {
        uid: u16,
        ppid: Option<u16>,
        pid: u16,
        location: Location,
    },
    Renice {
        pid: u16,
        new_nice: u16,
    },
    Kill {
        pid: u16,
    },
    Detach {
        pid: u16,
    },

    NewChallenge {
        challenge_type: &'a str,
        difficulty: u16,
        location: Location,
    },
    ChallengeSolved {
        pid: u16,
        location: Location,
    },

    ScoreUpdate {
        uid: u16,
        new_score: i64,
    },
    WalletUpdate {
        uid: u16,
        new_wallet: &'a Wallet,
    },
}

pub fn log_event(event: GameEvent) {
    println!("EVENT|{}", serde_json::to_string(&event).unwrap());
}
