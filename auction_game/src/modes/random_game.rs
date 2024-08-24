use crate::{models, modes};
use log::LevelFilter;
use models::state::GameState;
use modes::traits::{Game, PlayerController};
use rand::rngs::StdRng;
use rand::Rng;

struct RandomGame {
    pub game_id: String,
    level_filter: LevelFilter,
    controllers: Vec<Box<dyn PlayerController>>,
    bool_random_starting_player: bool,
    //     TODO: At some point also indicate the GUI Logger/Interface
}

impl RandomGame {
    pub fn new(
        game_id: String,
        level_filter: LevelFilter,
        controllers: Vec<Box<dyn PlayerController>>,
        bool_random_starting_player: bool,
    ) -> Self {
        RandomGame {
            game_id,
            level_filter,
            controllers,
            bool_random_starting_player,
        }
    }
}

impl Game for RandomGame {
    fn game_run() {
        init_logger(self.game_id, &self.game_id);
        let seed = string_to_seed(&self.game_id);
        let mut rng = StdRng::seed_from_u64(seed);
        let mut start_player: u8 = match self.bool_random_starting_player {
            false => 0,
            true => rng.gen_range(0..self.controllers.len()),
        };
        let mut no_players: u8 = self.controllers.len() as u8;
        let mut game_state = GameState::starting(no_players);
        info!("{game_state}");
        game_state.reveal_auction(GamePhase::Bid);
        let mut history: Vec<GameState> = Vec::with_capacity(100);
        while game_state.bid_phase_end() == false {
            // log game state
            history.push(game_state.clone());
            info!("{game_state}");
            let move_choices = game_state.legal_moves_bid(start_player);
            if let Some(random_item) = move_choices.choose(&mut rng) {
                info!(
                    "player {} chose to add {} to their bid",
                    start_player + 1,
                    random_item
                );
                game_state = game_state.generate_next_state_bid(start_player, *random_item);
            } else {
                panic!("The vector is empty.");
            }
            start_player = game_state.current_player();
        }
        info!("{game_state}");
        info!("");
        info!("===== Starting Sell Phase =====");
        info!("");
        game_state.reveal_auction(GamePhase::Sell);
        while game_state.sell_phase_end() == false {
            history.push(game_state.clone());
            info!("Before Sell {game_state}");
            let mut aggregate_sales: Vec<Property> = Vec::with_capacity(no_players as usize);
            for player in 0..no_players {
                // TODO: Reference controller traits
                let move_choices = game_state.legal_moves_sell(player);
                let mut rng = thread_rng();
                if let Some(random_item) = move_choices.choose(&mut rng) {
                    info!(
                        "player {} Randomly selected to Sell Property: {}",
                        player + 1,
                        random_item
                    );
                    aggregate_sales.push(*random_item);
                } else {
                    debug_assert!(false, "The vector is empty.");
                }
            }
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
