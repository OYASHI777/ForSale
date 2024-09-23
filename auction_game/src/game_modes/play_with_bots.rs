use crate::engines::algorithms::maxn_player::MaxNPlayer;
use crate::engines::controllers::greedy_baby::GreedyBaby;
use crate::engines::controllers::terminal_player::HumanPlayer;
use crate::game_modes::traits::Game;
use crate::{engines, models};
use ahash::AHashMap;
use engines::traits::PlayerController;
use helper::generation::string_to_seed;
use log::{info, LevelFilter};
use models::game_state::GameState;
use rand::rngs::StdRng;
use rand::SeedableRng;
use rand::{thread_rng, Rng};
use std::thread;
use std::time::Duration;

pub struct Play {
    pub game_id: String,
    level_filter: LevelFilter,
    // controllers: Vec<Box<dyn PlayerController>>,
    bool_random_starting_player: bool,
}

// TODO: Add controllers properly
impl Play {
    pub fn new(
        game_id: String,
        level_filter: LevelFilter,
        // controllers: Vec<Box<dyn PlayerController>>,
        bool_random_starting_player: bool,
    ) -> Self {
        Play {
            game_id,
            level_filter,
            // controllers,
            bool_random_starting_player,
        }
    }
}

impl Game for Play {
    fn game_run(&mut self) {
        // TODO: Deal with the logger not being able to be repeatedly initialized
        // init_logger(self.level_filter, &self.game_id);

        println!("Running Game ID: {}", self.game_id);
        // let seed = string_to_seed(&self.game_id);
        // let mut rng = StdRng::seed_from_u64(seed);
        let mut rng = thread_rng();
        let current_player: u8 = match self.bool_random_starting_player {
            false => 0,
            true => rng.gen_range(0..6u8), // TODO: Use self.controllers.len()
        };
        let no_players: u8 = 6;
        let mut controllers: AHashMap<u8, Box<dyn PlayerController>> =
            AHashMap::with_capacity(no_players as usize);
        controllers.insert(0, Box::new(HumanPlayer::new(0, "Brave Human".to_string())));
        for i in 1..no_players {
            controllers.insert(i, Box::new(GreedyBaby::new(i, format!("P{i}").to_string())));
        }
        // TODO: Organise human and greedy baby controllers
        let mut human = HumanPlayer::new(0, "Brave Human".to_string());
        let mut greedy_baby = GreedyBaby::new(0, "ENGINE".to_string());
        let mut game_state = GameState::starting(no_players, current_player);
        println!("GameState: {}", game_state);
        game_state.reveal_auction();
        let mut last_round = game_state.round_no();
        while game_state.bid_phase_end() == false {
            println!("{game_state}");
            if game_state.round_no() > last_round {
                last_round = game_state.round_no();
                game_state.reveal_auction();
                continue;
            }
            let current_player = game_state.current_player();
            println!("It's Bot {}'s turn", current_player + 1);
            let mut best_move: u8 = 0;
            if let Some(player_control) = controllers.get_mut(&current_player) {
                best_move = player_control.decision(&game_state);
            }
            if game_state.turn_no() > 1 && game_state.current_player() != 0 {
                thread::sleep(Duration::from_secs(3));
            }
            println!("Player: {} chose to do: {}", current_player + 1, best_move);
            game_state = game_state.generate_next_state_bid(current_player, best_move);
        }
        println!("{game_state}");

        // TODO: Make first half only and second half only
        // let end_scores = MaxNPlayer::round_score_function(&game_state);
        // println!("Ending Score is: {:?}", end_scores);
        // let rank = find_ranking(&end_scores);
        // println!("Your rank was {}!", rank);
        // // let output = player.maximax_round(&game_state, 1, true, 1);
        // // info!("Best move is: {}", output);
        // println!("END");

        println!("");
        println!("===== Starting Sell Phase =====");
        println!("");
        game_state.reveal_auction();
        while game_state.game_end() == false {
            let mut aggregate_sales = match game_state.auction_end() {
                true => {
                    println!("Before Sell {game_state}");
                    vec![0; 6]
                }
                false => {
                    println!("Before Sell {game_state}");
                    // TODO: Consider moving batch decision
                    let mut temp = greedy_baby.batch_decision(&game_state);
                    let action = human.decision(&game_state);
                    temp[0] = action;
                    println!("Properties Chosen by Players were: {:?}", temp);
                    temp
                }
            };
            game_state = game_state.generate_next_state_sell(aggregate_sales);
        }
        println!("{game_state}");
        println!("Game has concluded!");
        println!("\n{}", game_state.tally_game_score());
    }
}
fn find_ranking(values: &Vec<f32>) -> usize {
    let mut sorted_values = values.clone();
    sorted_values.sort_by(|a, b| b.partial_cmp(a).unwrap_or(std::cmp::Ordering::Equal));

    sorted_values.iter().position(|&x| x == values[0]).unwrap() + 1
}
