use crate::models::game_state::GameState;

pub trait PlayerController {
    fn nickname(&self) -> String;
    fn decision(&mut self, game_state: &GameState) -> u8;
    fn batch_decision(&mut self, game_state: &GameState) -> Vec<u8>;
}
