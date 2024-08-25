use crate::models::game_state::GameState;

pub trait PlayerController {
    fn nickname(&self) -> String;
    fn decision(&mut self, game_state: &GameState) -> u8;
}
