use auction_game::engines::controllers::maxn_player::MaxNPlayer;
use auction_game::engines::traits::PlayerController;
use auction_game::game_modes::standard::StandardGame;
use auction_game::game_modes::traits::Game;
use auction_game::models::enums::GamePhase;
use auction_game::models::game_state::GameState;
use helper::logger::init_logger;
use log::{info, LevelFilter};

fn main() {
    init_logger(LevelFilter::Debug, "test_maximax");
    let no_players: u8 = 6;
    let mut game_state = GameState::starting(no_players, 0);
    game_state.reveal_auction(GamePhase::Bid);
    info!("Initial GameState: {}", game_state);
    let mut player = MaxNPlayer::new(0, "Bob".to_string());
    player.maximax_depth(&game_state, 3);
}
