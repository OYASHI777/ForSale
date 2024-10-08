use crate::game_modes::traits::Game;
use crate::models::enums::{Coins, Property};
use crate::{engines, models};
use engines::traits::PlayerController;
use helper::generation::string_to_seed;
use helper::logger::init_logger;
use log::{info, LevelFilter};
use models::game_state::GameState;
use rand::rngs::StdRng;
use rand::Rng;
use rand::SeedableRng;

pub struct StandardGame {
    pub game_id: String,
    level_filter: LevelFilter,
    controllers: Vec<Box<dyn PlayerController>>,
    bool_random_starting_player: bool,
    //     TODO: At some point also indicate the GUI Logger/Interface
}

impl StandardGame {
    pub fn new(
        game_id: String,
        level_filter: LevelFilter,
        controllers: Vec<Box<dyn PlayerController>>,
        bool_random_starting_player: bool,
    ) -> Self {
        StandardGame {
            game_id,
            level_filter,
            controllers,
            bool_random_starting_player,
        }
    }
}

impl Game for StandardGame {
    fn game_run(&mut self) {
        init_logger(self.level_filter, &self.game_id);
        let seed = string_to_seed(&self.game_id);
        let mut rng = StdRng::seed_from_u64(seed);
        let mut current_player: u8 = match self.bool_random_starting_player {
            false => 0,
            true => rng.gen_range(0..self.controllers.len() as u8),
        };
        let no_players: u8 = self.controllers.len() as u8;
        let mut game_state = GameState::starting(no_players, current_player);
        info!(
            "Starting game: {}|First player is player {}",
            self.game_id,
            current_player + 1
        );
        info!("{game_state}");
        game_state.reveal_auction();
        let mut history: Vec<GameState> = Vec::with_capacity(100);

        while game_state.bid_phase_end() == false {
            history.push(game_state.clone());
            info!("{game_state}");
            let move_choice: Coins =
                self.controllers[current_player as usize].decision(&game_state);
            info!(
                "player {} chose to add {} to their bid",
                current_player + 1,
                move_choice
            );
            game_state = game_state.generate_next_state_bid(current_player, move_choice);
            current_player = game_state.current_player();
        }
        info!("{game_state}");
        info!("");
        info!("===== Starting Sell Phase =====");
        info!("");
        game_state.reveal_auction();
        while game_state.game_end() == false {
            history.push(game_state.clone());
            info!("Before Sell {game_state}");
            let aggregate_sales = match game_state.auction_end() {
                true => {
                    vec![0; 6]
                }
                false => {
                    let mut temp = Vec::with_capacity(no_players as usize);
                    for player in 0..no_players {
                        let move_choice: Property =
                            self.controllers[player as usize].decision(&game_state);
                        info!(
                            "player {} Randomly selected to Sell Property: {}",
                            player + 1,
                            move_choice
                        );
                        temp.push(move_choice);
                    }
                    temp
                }
            };
            game_state = game_state.generate_next_state_sell(aggregate_sales);
        }
        info!("{game_state}");
        info!(
            "\n ===== Auctions have closed after {} turns =====",
            history.len()
        );
        info!("\n{}", game_state.tally_game_score());
    }
}
