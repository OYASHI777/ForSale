use crate::engines::traits::PlayerController;
use crate::models::game_state::GameState;
use rand::rngs::ThreadRng;
use rand::seq::IndexedRandom;
use rand::thread_rng;

pub struct MinimaxPlayer {
    id: u8,
    nickname: String,
    rng: ThreadRng,
}

impl MinimaxPlayer {
    pub fn new(id: u8, nickname: String) -> Self {
        let rng = thread_rng();
        MinimaxPlayer { id, nickname, rng }
    }
}

impl PlayerController for MinimaxPlayer {
    fn nickname(&self) -> String {
        self.nickname.clone()
    }
    fn decision(&mut self, game_state: &GameState) -> u8 {
        let legal_moves: Vec<u8> = game_state.legal_moves(self.id);
        debug_assert!(
            legal_moves.len() > 0,
            "Legal Moves Provided for player {} is empty",
            self.id
        );
        *legal_moves.choose(&mut self.rng).unwrap()
    }
}
