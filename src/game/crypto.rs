use murmurhash3::murmurhash3_x86_32;
use rand::random;
use serde::Serialize;
use sha2::{Digest, Sha256};

pub const CRYPTO_TYPES: usize = std::mem::variant_count::<CryptoCurrency>();

pub enum CryptoCurrency {
    DogeCoin,
    StarSleepShortage,
    Ethereum,
    BitCoin,
    CrabCoin,
    Ｅｘｐｌｏｓｉｏｎ,
}

#[derive(Default, Serialize)]
pub struct Wallet {
    assets: [i64; CRYPTO_TYPES],
}

impl Wallet {
    pub fn get_currency(&self, currency: CryptoCurrency) -> i64 {
        self.assets[currency as usize]
    }

    pub fn add_currency(mut self, currency: CryptoCurrency, amount: i64) -> Self {
        let idx = currency as usize;
        self.assets[idx] += amount;
        self
    }

    pub fn can_afford(&self, cost: &Wallet) -> bool {
        self.assets
            .iter()
            .zip(cost.assets.iter())
            .all(|(x, y)| x >= y || *y <= 0)
    }

    pub fn convert_to_score(&self) -> i64 {
        let [doge, sleep, eth, bitcoin, crab, _] = self.assets;
        let base_score = doge * 3 + sleep * -1 + eth * 420 + bitcoin * 35_995;
        let multiplier = 1.0 + 0.01 * crab as f64;
        (base_score as f64 * multiplier).round() as i64
    }

    pub fn get_newbie_welcome_pack() -> Self {
        wallet!(
            DogeCoin: 1337,
            StarSleepShortage: -690,
            Ethereum: 128,
            BitCoin: 0,
            CrabCoin: 0,
            Ｅｘｐｌｏｓｉｏｎ: 1,
        )
    }
}

impl std::ops::AddAssign<&Wallet> for Wallet {
    fn add_assign(&mut self, rhs: &Wallet) {
        self.assets = std::array::from_fn(|i| self.assets[i] + rhs.assets[i]);
    }
}

impl std::ops::SubAssign<&Wallet> for Wallet {
    fn sub_assign(&mut self, rhs: &Wallet) {
        self.assets = std::array::from_fn(|i| self.assets[i] - rhs.assets[i]);
    }
}

macro_rules! wallet {
    ($($currency:ident: $amount:expr),* $(,)?) => {
        Wallet::default()
            $(.add_currency($crate::game::crypto::CryptoCurrency::$currency, $amount))*
    }
}
pub(crate) use wallet;

pub trait CryptoChallenge {
    fn get_numeric_id(&self) -> u16;
    fn get_name(&self) -> &'static str;
    fn generate(&mut self, difficulty: i64);
    fn get_reward(&self) -> Wallet;
    fn get_difficulty(&self) -> u16;
    fn get_challenge_data(&self) -> Vec<u8>;
    fn verify(&self, nonce: (u16, u16, u16, u16)) -> bool;
}

pub fn generate_crypto_challenge(name: &str, difficulty: i64) -> Option<Box<dyn CryptoChallenge>> {
    let mut challenge: Box<dyn CryptoChallenge> = match name {
        "bed" => Box::new(BedChallenge::default()),
        "dog" => Box::new(DogChallenge::default()),
        "ether" => Box::new(EtherChallenge::default()),
        "btc" => Box::new(BabyBtcChallenge::default()),
        "crab" => Box::new(CrabChallenge::default()),
        _ => return None,
    };
    challenge.generate(difficulty);
    Some(challenge)
}

#[derive(Default)]
pub struct BedChallenge {
    difficulty: i64,
}

impl CryptoChallenge for BedChallenge {
    fn get_numeric_id(&self) -> u16 {
        0xbed
    }
    fn get_name(&self) -> &'static str {
        "bed"
    }
    fn generate(&mut self, difficulty: i64) {
        self.difficulty = difficulty;
    }
    fn get_reward(&self) -> Wallet {
        wallet!(StarSleepShortage: -self.difficulty)
    }
    fn get_difficulty(&self) -> u16 {
        self.difficulty as _
    }
    fn get_challenge_data(&self) -> Vec<u8> {
        vec![]
    }
    fn verify(&self, nonce: (u16, u16, u16, u16)) -> bool {
        nonce.0 == 0 && nonce.1 == 1 && nonce.2 == 2 && nonce.3 == 3
    }
}

#[derive(Default)]
pub struct DogChallenge {
    difficulty: i64,
    challenge: (u16, u16, u16, u16),
}

impl CryptoChallenge for DogChallenge {
    fn get_numeric_id(&self) -> u16 {
        0x420
    }
    fn get_name(&self) -> &'static str {
        "dog"
    }
    fn generate(&mut self, difficulty: i64) {
        self.difficulty = difficulty;
        self.challenge = random();
    }
    fn get_reward(&self) -> Wallet {
        wallet!(
            DogeCoin: self.difficulty,
            StarSleepShortage: 1,
        )
    }
    fn get_difficulty(&self) -> u16 {
        self.difficulty as _
    }
    fn get_challenge_data(&self) -> Vec<u8> {
        [
            self.challenge.0.to_le_bytes(),
            self.challenge.1.to_le_bytes(),
            self.challenge.2.to_le_bytes(),
            self.challenge.3.to_le_bytes(),
        ]
        .concat()
    }
    fn verify(&self, nonce: (u16, u16, u16, u16)) -> bool {
        nonce == self.challenge
    }
}

#[derive(Default)]
pub struct EtherChallenge {
    difficulty: i64,
    challenge: u16,
}

impl CryptoChallenge for EtherChallenge {
    fn get_numeric_id(&self) -> u16 {
        0x1337
    }
    fn get_name(&self) -> &'static str {
        "ether"
    }
    fn generate(&mut self, difficulty: i64) {
        self.difficulty = difficulty;
        self.challenge = random::<u16>() | 1;
    }
    fn get_reward(&self) -> Wallet {
        wallet!(
            Ethereum: self.difficulty,
            StarSleepShortage: 3,
        )
    }
    fn get_difficulty(&self) -> u16 {
        self.difficulty as _
    }
    fn get_challenge_data(&self) -> Vec<u8> {
        self.challenge.to_le_bytes().into()
    }
    fn verify(&self, nonce: (u16, u16, u16, u16)) -> bool {
        nonce.0.wrapping_mul(self.challenge) == 1 && nonce.1 == 0 && nonce.2 == 0 && nonce.3 == 0
    }
}

#[derive(Default)]
pub struct BabyBtcChallenge {
    challenge: [u8; 32],
}

impl CryptoChallenge for BabyBtcChallenge {
    fn get_numeric_id(&self) -> u16 {
        0xb7c
    }
    fn get_name(&self) -> &'static str {
        "btc"
    }
    fn generate(&mut self, _difficulty: i64) {
        self.challenge = random();
    }
    fn get_reward(&self) -> Wallet {
        wallet!(
            BitCoin: 1,
        )
    }
    fn get_difficulty(&self) -> u16 {
        1
    }
    fn get_challenge_data(&self) -> Vec<u8> {
        self.challenge.into()
    }
    fn verify(&self, nonce: (u16, u16, u16, u16)) -> bool {
        let data = [
            nonce.0.to_le_bytes(),
            nonce.1.to_le_bytes(),
            nonce.2.to_le_bytes(),
            nonce.3.to_le_bytes(),
        ]
        .concat();
        Sha256::digest(self.challenge)[..data.len()] == data
    }
}

#[derive(Default)]
pub struct CrabChallenge {
    seed: u32,
    hash: u32,
    challenge: [u8; 32],
}

impl CryptoChallenge for CrabChallenge {
    fn get_numeric_id(&self) -> u16 {
        '🦀' as u16
    }
    fn get_name(&self) -> &'static str {
        "crab"
    }
    fn generate(&mut self, _difficulty: i64) {
        self.challenge = random();
        self.seed = random();
        self.hash = random();
    }
    fn get_reward(&self) -> Wallet {
        wallet!(
            CrabCoin: 1,
        )
    }
    fn get_difficulty(&self) -> u16 {
        1
    }
    fn get_challenge_data(&self) -> Vec<u8> {
        [
            &self.seed.to_le_bytes()[..],
            &self.hash.to_le_bytes()[..],
            &self.challenge[..],
        ]
        .concat()
    }
    fn verify(&self, nonce: (u16, u16, u16, u16)) -> bool {
        let mut data = [
            nonce.0.to_le_bytes(),
            nonce.1.to_le_bytes(),
            nonce.2.to_le_bytes(),
            nonce.3.to_le_bytes(),
        ]
        .concat();
        data.extend(self.challenge);
        murmurhash3_x86_32(&data, self.seed) == self.hash
    }
}
