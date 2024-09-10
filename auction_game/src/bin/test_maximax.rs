use auction_game::engines::controllers::maxn_player::MaxNPlayer;
use auction_game::models::game_state::GameState;
use helper::logger::init_logger;
use log::{info, LevelFilter};

fn main() {
    // TODO: Test for subsequent rounds too
    test_maximax_round()
}
fn test_maximax_round() {
    // TODO: Log deepening, states added
    init_logger(LevelFilter::Info, "test_maximax");
    let no_players: u8 = 6;
    let mut game_state = GameState::starting(no_players, 0);
    game_state.reveal_auction_manual(vec![1, 2, 3, 4, 5, 30]);
    info!("Initial GameState: {}", game_state);
    let mut player = MaxNPlayer::new(0, "Bob".to_string());
    let output = player.maximax_round(&game_state, 1, true, 1, true);
    info!("Best move is: {}", output);
    info!("END");
}
