use auction_game::engines::controllers::random_player::RandomPlayer;
use auction_game::engines::traits::PlayerController;
use auction_game::game_modes::standard::StandardGame;
use auction_game::game_modes::traits::Game;
use auction_game::models::enums::{GamePhase, Property};
use auction_game::models::state::GameState;
use helper::logger::init_logger;
use log::{info, LevelFilter};
use rand::prelude::IndexedRandom;
use rand::thread_rng;

// TODO: Add to github
fn main() {
    let no_players: u8 = 6;
    let mut controllers: Vec<Box<dyn PlayerController>> = Vec::with_capacity(no_players as usize);
    for id in 0..no_players as usize {
        let controller: Box<RandomPlayer> =
            Box::new(RandomPlayer::new(id as u8, format!("Player_{id}")));
        controllers.push(controller);
    }
    let mut game = StandardGame::new(
        "random_game".to_string(),
        LevelFilter::Info,
        controllers,
        true,
    );
    game.game_run();
    // init_logger(LevelFilter::Debug, "rand_game");
    // let mut start_player: u8 = 0;
    // let no_players: u8 = 6;
    // let mut game_state = GameState::starting(no_players);
    // info!("{game_state}");
    // game_state.reveal_auction(GamePhase::Bid);
    // let mut history: Vec<GameState> = Vec::with_capacity(100);
    // while game_state.bid_phase_end() == false {
    //     // log game state
    //     history.push(game_state.clone());
    //     info!("{game_state}");
    //     // TODO: Replace with some trait
    //     let move_choices = game_state.legal_moves_bid(start_player);
    //     let mut rng = thread_rng();
    //     // TODO: Include all print statements in another file
    //     if let Some(random_item) = move_choices.choose(&mut rng) {
    //         info!(
    //             "player {} chose to add {} to their bid",
    //             start_player + 1,
    //             random_item
    //         );
    //         game_state = game_state.generate_next_state_bid(start_player, *random_item);
    //     } else {
    //         panic!("The vector is empty.");
    //     }
    //     start_player = game_state.current_player();
    // }
    // info!("{game_state}");
    // info!("");
    // info!("===== Starting Sell Phase =====");
    // info!("");
    // game_state.reveal_auction(GamePhase::Sell);
    // while game_state.sell_phase_end() == false {
    //     history.push(game_state.clone());
    //     info!("Before Sell {game_state}");
    //     // TODO: Replace with some trait
    //     let mut aggregate_sales: Vec<Property> = Vec::with_capacity(no_players as usize);
    //     for player in 0..no_players {
    //         let move_choices = game_state.legal_moves_sell(player);
    //         let mut rng = thread_rng();
    //         if let Some(random_item) = move_choices.choose(&mut rng) {
    //             info!(
    //                 "player {} Randomly selected to Sell Property: {}",
    //                 player + 1,
    //                 random_item
    //             );
    //             aggregate_sales.push(*random_item);
    //         } else {
    //             debug_assert!(false, "The vector is empty.");
    //         }
    //     }
    //     game_state = game_state.generate_next_state_sell(aggregate_sales);
    // }
    // info!("{game_state}");
    // info!("\n ===== Auctions have close =====");
    // info!("\n{}", game_state.tally_game_score());
}
