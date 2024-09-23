use auction_game::engines::controllers::greedy_baby::GreedyBaby;
use auction_game::engines::controllers::random_player::RandomPlayer;
use auction_game::engines::controllers::terminal_player::HumanPlayer;
use auction_game::engines::traits::PlayerController;
use auction_game::game_modes::standard::StandardGame;
use auction_game::game_modes::traits::Game;
use auction_game::models::enums::Coins;
use auction_game::models::game_state::GameState;
use helper::generation::string_to_seed;
use helper::logger::init_logger;
use log::LevelFilter::Debug;
use log::{info, LevelFilter};

fn main() {
    // TODO: Run greedy baby with random first half
    init_logger(Debug, "CFR_TEST");
    let mut greedy_baby = GreedyBaby::new(0, "BOB".to_string());

    let no_players: u8 = 6;
    let mut current_player: u8 = 0;
    let mut game_state = GameState::starting(no_players, current_player);
    let mut controllers: Vec<Box<dyn PlayerController>> = Vec::with_capacity(no_players as usize);
    for id in 0..no_players as usize {
        let controller: Box<RandomPlayer> =
            Box::new(RandomPlayer::new(id as u8, format!("Player_{id}")));
        controllers.push(controller);
    }
    info!(
        "Starting CFR Test |First player is player {}",
        current_player + 1
    );
    info!("{game_state}");
    game_state.reveal_auction();
    let mut history: Vec<GameState> = Vec::with_capacity(100);

    while game_state.bid_phase_end() == false {
        history.push(game_state.clone());
        info!("{game_state}");
        let move_choice: Coins = controllers[current_player as usize].decision(&game_state);
        info!(
            "player {} chose to add {} to their bid",
            current_player + 1,
            move_choice
        );
        game_state = game_state.generate_next_state_bid(current_player, move_choice);
        current_player = game_state.current_player();
    }
    info!("{game_state}");
    // Single
    // info!("");
    // info!("===== Starting Sell Phase =====");
    // info!("");
    // game_state.reveal_auction();
    // while game_state.game_end() == false {
    //     history.push(game_state.clone());
    //     info!("Before Sell {game_state}");
    //     let aggregate_sales = match game_state.auction_end() {
    //         true => {
    //             vec![0; 6]
    //         }
    //         false => greedy_baby.batch_decision(&game_state),
    //     };
    //     game_state = game_state.generate_next_state_sell(aggregate_sales);
    //     break;
    // }
    // info!("{game_state}");
    // info!(
    //     "\n ===== Auctions have closed after {} turns =====",
    //     history.len()
    // );
    // info!("\n{}", game_state.tally_game_score());
    // Multi
    info!("");
    info!("===== Starting Sell Phase =====");
    info!("");
    let mut human = HumanPlayer::new(0, "Brave Human".to_string());
    // game_state.reveal_auction();
    println!("Before Sell {game_state}");
    while game_state.game_end() == false {
        println!("Before Sell {game_state}");
        let mut aggregate_sales = match game_state.auction_end() {
            true => {
                vec![0; 6]
            }
            false => {
                let mut temp = greedy_baby.batch_decision(&game_state);
                let action = human.decision(&game_state);
                temp[0] = action;
                temp
            }
        };
        game_state = game_state.generate_next_state_sell(aggregate_sales);
    }
    println!("{game_state}");
    println!("Game has concluded!");
    println!("\n{}", game_state.tally_game_score());
}
