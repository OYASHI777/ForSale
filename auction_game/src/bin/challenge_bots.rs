use auction_game::game_modes::play_with_bots::Play;
use auction_game::game_modes::traits::Game;
use helper::logger::init_logger;
use log::LevelFilter;

fn main() {
    // TODO: Test searching more than 1 node

    let mut id: u32 = 0;
    init_logger(LevelFilter::Info, "Bot Challenge");
    while id < 1 {
        let mut game = Play::new(
            format!("Test_{id}").to_string(),
            LevelFilter::Info,
            true,
            true,
        );
        game.game_run();
        id += 1;
    }
}
