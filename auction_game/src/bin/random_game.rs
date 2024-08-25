use auction_game::engines::controllers::random_player::RandomPlayer;
use auction_game::engines::traits::PlayerController;
use auction_game::game_modes::standard::StandardGame;
use auction_game::game_modes::traits::Game;
use log::LevelFilter;

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
        LevelFilter::Debug,
        controllers,
        true,
    );
    game.game_run();
}
