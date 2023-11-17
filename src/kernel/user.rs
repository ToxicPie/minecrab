use crate::game::crypto::{CryptoCurrency, Wallet};
use crate::game::replay::{log_event, GameEvent};

pub struct User {
    pub uid: u16,
    pub initd_pid: Option<u16>,
    pub score: i64,
    pub num_processes: usize,
    pub wallet: Wallet,
}

impl User {
    pub fn compute_sleep_debt(&self, cycles: usize) -> usize {
        let sleep_debt = self.wallet.get_currency(CryptoCurrency::StarSleepShortage);
        cycles - sleep_debt.clamp(0, cycles as i64 * 3 / 4) as usize
    }

    pub fn convert_wallet_to_score(&mut self) {
        let score = self.wallet.convert_to_score();
        self.wallet = Wallet::default();
        self.score += score;
        log_event(GameEvent::WalletUpdate {
            uid: self.uid,
            new_wallet: &self.wallet,
        });
        log_event(GameEvent::ScoreUpdate {
            uid: self.uid,
            new_score: self.score,
        });
    }
}
