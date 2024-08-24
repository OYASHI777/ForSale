use std::fmt;

#[derive(Copy, Clone, Eq, PartialEq)]
pub enum GamePhase {
    Bid,
    Sell,
}

impl fmt::Display for GamePhase {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let phase_name = match self {
            GamePhase::Bid => "Bid",
            GamePhase::Sell => "Sell",
        };
        write!(f, "{}", phase_name)
    }
}

pub type Coins = u8;
pub type Property = u8;
pub type Check = u8;
pub type Player = u8;
