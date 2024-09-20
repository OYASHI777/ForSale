use crate::engines::controllers::counterfactual_regret::CFR;
use crate::engines::controllers::maxn_player::MaxNPlayer;
use crate::engines::traits::PlayerController;
use crate::models::enums::GamePhase;
use crate::models::game_state::GameState;

pub struct GreedyBaby {
    id: u8,
    nickname: String,
    maxn_controller: MaxNPlayer,
    cfr_controller: CFR,
}

impl GreedyBaby {
    pub fn new(id: u8, nickname: String) -> Self {
        // TODO: consider making bool_print inputs?
        let maxn_controller = MaxNPlayer::new(id, nickname.clone(), false, false);
        let cfr_controller = CFR::new(false);
        GreedyBaby {
            id,
            nickname,
            maxn_controller,
            cfr_controller,
        }
    }
}

impl PlayerController for GreedyBaby {
    fn nickname(&self) -> String {
        self.nickname.clone()
    }
    fn decision(&mut self, game_state: &GameState) -> u8 {
        self.maxn_controller.maximax_round(&game_state, 1, false, 0)
    }
    fn batch_decision(&mut self, game_state: &GameState) -> Vec<u8> {
        self.cfr_controller.find_nash(game_state, 20000);
        let mut aggregate_actions: Vec<u8> = Vec::with_capacity(game_state.no_players() as usize);
        for player in 0..game_state.no_players() {
            let action = self.cfr_controller.get_mixed_strategy(game_state, player);
            aggregate_actions.push(action);
        }
        aggregate_actions
    }
}
