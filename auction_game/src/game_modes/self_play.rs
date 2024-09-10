use crate::engines::controllers::maxn_player::MaxNPlayer;
use crate::game_modes::traits::Game;
use crate::models::enums::{Coins, Property};
use crate::{engines, models};
use ahash::AHashMap;
use engines::traits::PlayerController;
use helper::generation::string_to_seed;
use helper::logger::init_logger;
use log::{info, LevelFilter};
use models::game_state::GameState;
use rand::rngs::StdRng;
use rand::Rng;
use rand::SeedableRng;

pub struct SelfPlay {
    pub game_id: String,
    level_filter: LevelFilter,
    // controllers: Vec<Box<dyn PlayerController>>,
    bool_random_starting_player: bool,
    //     TODO: At some point also indicate the GUI Logger/Interface
}

// TODO: Add controllers properly
impl SelfPlay {
    pub fn new(
        game_id: String,
        level_filter: LevelFilter,
        // controllers: Vec<Box<dyn PlayerController>>,
        bool_random_starting_player: bool,
    ) -> Self {
        SelfPlay {
            game_id,
            level_filter,
            // controllers,
            bool_random_starting_player,
        }
    }
}

impl Game for SelfPlay {
    fn game_run(&mut self) {
        // TODO: Deal with the logger not being able to be repeatedly initialized
        // init_logger(self.level_filter, &self.game_id);
        info!("Running Game ID: {}", self.game_id);
        let seed = string_to_seed(&self.game_id);
        let mut rng = StdRng::seed_from_u64(seed);
        let mut current_player: u8 = match self.bool_random_starting_player {
            false => 0,
            true => rng.gen_range(0..6 as u8), // TODO: Use self.controllers.len()
        };
        let no_players: u8 = 6;
        let mut controllers: AHashMap<u8, MaxNPlayer> =
            AHashMap::with_capacity(no_players as usize);
        for i in 0..no_players {
            controllers.insert(i, MaxNPlayer::new(i, format!("P{i}").to_string()));
        }
        let mut game_state = GameState::starting(no_players, current_player);
        info!("GameState: {}", game_state);
        game_state.reveal_auction();
        let mut last_round = game_state.round_no();
        while game_state.bid_phase_end() == false {
            info!("{game_state}");
            if game_state.round_no() > last_round {
                last_round = game_state.round_no();
                game_state.reveal_auction();
                continue;
            }
            let current_player = game_state.current_player();
            let mut best_move: u8 = 0;
            if let Some(player_control) = controllers.get_mut(&current_player) {
                let rounds_param: u8 = match game_state.round_no() {
                    0 => 1,
                    1 => 1,
                    2 => 1,
                    3 => 2,
                    4 => 1,
                    _ => 1,
                };
                best_move = player_control.maximax_round(&game_state, rounds_param, false, 0, true);
            }
            info!("Player: {} chose to do: {}", current_player + 1, best_move);
            game_state = game_state.generate_next_state_bid(current_player, best_move);
        }
        info!("{game_state}");
        // let output = player.maximax_round(&game_state, 1, true, 1);
        // info!("Best move is: {}", output);
        info!("END");
    }
}
