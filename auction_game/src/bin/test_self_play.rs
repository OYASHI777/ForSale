use ahash::AHashMap;
use auction_game::engines::controllers::maxn_player::MaxNPlayer;
use auction_game::models::enums::GamePhase;
use auction_game::models::game_state::GameState;
use helper::logger::init_logger;
use log::{info, LevelFilter};

fn main() {
    // test_self_play();
    replicate_issue();
}

fn test_self_play() {
    // TODO: Log deepening, states added
    init_logger(LevelFilter::Info, "test_self_play");
    let no_players: u8 = 6;
    let mut controllers: AHashMap<u8, MaxNPlayer> = AHashMap::with_capacity(no_players as usize);
    for i in 0..no_players {
        controllers.insert(i, MaxNPlayer::new(i, format!("P{i}").to_string()));
    }
    let mut game_state = GameState::starting(no_players, 0);
    info!("GameState: {}", game_state);
    game_state.reveal_auction_manual(vec![1, 2, 3, 4, 5, 30]);
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
            best_move = player_control.maximax_round(&game_state, 1, false, 0);
        }
        info!("Player: {} chose to do: {}", current_player, best_move);
        game_state = game_state.generate_next_state_bid(current_player, best_move);
    }
    info!("{game_state}");
    // let output = player.maximax_round(&game_state, 1, true, 1);
    // info!("Best move is: {}", output);
    info!("END");
}

fn replicate_issue() {
    // TODO: Log deepening, states added
    // SO basically if the next round to be considered after initial state is a terminal node, the node wont be in score because score is only added when propagating and not for terminal nodes
    init_logger(LevelFilter::Info, "maxn_issue0");
    let no_players: u8 = 6;
    let mut controllers: AHashMap<u8, MaxNPlayer> = AHashMap::with_capacity(no_players as usize);
    for i in 0..no_players {
        controllers.insert(i, MaxNPlayer::new(i, format!("P{i}").to_string()));
    }
    let mut game_state = GameState::starting(no_players, 0);
    info!("GameState: {}", game_state);
    game_state.reveal_auction_manual(vec![1, 2, 3, 4, 5, 30]);
    game_state = game_state.generate_next_state_bid(0, 9);
    game_state = game_state.generate_next_state_bid(1, 0);
    game_state = game_state.generate_next_state_bid(2, 0);
    game_state = game_state.generate_next_state_bid(3, 0);
    game_state = game_state.generate_next_state_bid(4, 0);
    info!("Initial GameState: {}", game_state);
    let mut player = MaxNPlayer::new(0, "Bob".to_string());
    let output = player.maximax_round(&game_state, 1, true, 1);
    info!("Best move is: {}", output);
    info!("END");
}
