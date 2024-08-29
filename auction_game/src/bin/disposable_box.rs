use auction_game::models::game_state::GameState;

fn main() {
    let mm = GameState::starting(6, 0);
    println!("{}", mm.get_encoding());
    println!("{}", mm.get_encoding());
    println!("{}", mm.get_encoding());
    println!("{}", mm.get_encoding());
}
