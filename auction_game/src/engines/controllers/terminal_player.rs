use crate::engines::traits::PlayerController;
use crate::models::game_state::GameState;
use std::io;

pub struct HumanPlayer {
    id: u8,
    nickname: String,
}

impl PlayerController for HumanPlayer {
    fn nickname(&self) -> String {
        self.nickname.clone()
    }

    fn decision(&mut self, game_state: &GameState) -> u8 {
        let legal_moves: Vec<u8> = game_state.legal_moves(self.id);
        debug_assert!(
            legal_moves.len() > 0,
            "Legal Moves Provided for player {} is empty",
            self.id
        );

        // Prompt the user to input their choice
        println!("{}, it's your turn!", self.nickname);
        println!("Available moves: {:?}", legal_moves);
        println!("Enter your choice:");

        loop {
            let mut input = String::new();
            io::stdin()
                .read_line(&mut input)
                .expect("Failed to read line");

            match input.trim().parse() {
                Ok(choice) if legal_moves.contains(&choice) => return choice,
                _ => {
                    println!("Invalid choice. Please enter a valid move from the available moves.");
                }
            }
        }
    }
    fn batch_decision(&mut self, game_state: &GameState) -> Vec<u8> {
        todo!()
    }
}

impl HumanPlayer {
    pub fn new(id: u8, nickname: String) -> Self {
        HumanPlayer { id, nickname }
    }
}
