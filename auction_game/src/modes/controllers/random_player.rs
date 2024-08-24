use crate::modes::traits::PlayerController;

struct RandomPlayer {
    nickname: String,
    //     TODO: Add rng
}

impl RandomPlayer {
    pub fn new(nickname: String) -> Self {
        RandomPlayer { nickname }
    }
}

impl PlayerController for RandomPlayer {
    fn nickname(&self) -> String {
        self.nickname.clone()
    }
    //     TODO: Add choose action
}
