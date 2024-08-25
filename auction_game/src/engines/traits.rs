use crate::models::state::GameState;

pub trait PlayerController {
    fn nickname(&self) -> String;
    fn decision(&mut self, game_state: &GameState) -> u8;
}
