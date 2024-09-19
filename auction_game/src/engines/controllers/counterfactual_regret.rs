use crate::game_modes::traits::Game;
use crate::models::enums::GamePhase;
use crate::models::game_state::GameState;
use ahash::AHashMap;
use bimap::BiMap;

pub struct CFR {
    move_map: AHashMap<String, Vec<BiMap<usize, u8>>>,
    strategy: AHashMap<String, Vec<Vec<f32>>>,
    q_values: AHashMap<String, Vec<Vec<f32>>>,
    buffer: Vec<GameState>,
}

impl CFR {
    pub fn new() -> Self {
        let move_map: AHashMap<String, Vec<BiMap<usize, u8>>> = AHashMap::with_capacity(1);
        let strategy: AHashMap<String, Vec<Vec<f32>>> = AHashMap::with_capacity(1);
        let q_values: AHashMap<String, Vec<Vec<f32>>> = AHashMap::with_capacity(1);
        let buffer: Vec<GameState> = Vec::with_capacity(1000);
        CFR {
            move_map,
            strategy,
            q_values,
            buffer,
        }
    }

    pub fn initialise_node(&mut self, game_state: &GameState) {
        if game_state.game_phase() == GamePhase::Sell {
            let path = game_state.get_path_encoding();
            let no_players = game_state.no_players();
            let initial_strategies: Vec<Vec<f32>> =
                vec![vec![0.0; no_players as usize]; no_players as usize];
            self.strategy
                .insert(path.clone(), initial_strategies.clone());
            self.q_values.insert(path.clone(), initial_strategies);
            for player in 0..no_players {
                let player_legal_moves = game_state.legal_moves(player);
                let mut player_move_map: BiMap<usize, u8> = BiMap::with_capacity(10);
                for (index, action) in player_legal_moves.iter().enumerate() {
                    player_move_map.insert(index, *action);
                }
            }
        } else {
            todo!("Focused on Greedy Baby for now");
        }
    }

    pub fn add_game_state(&mut self, game_state: GameState) {
        self.buffer.push(game_state);
    }

    pub fn sample_strategy(&self) {
        //     Sample from 0 to 1
        //     Iteratively jump over each strategy
        // If end is reached and sum is 0, choose the index corresponding to the nearest
        todo!("Integrate this");
    }

    pub fn iterate() {
        // Do one for each player
        todo!("One iteration");
    }

    pub fn find_nash() {
        // For each player, get regret
        //      For each legal move
        //      Simulate all other moves based on strategy
        //      Update regret
        // For all q_values update the strategy
        todo!("Iterate many times");
    }
}

// TODO: Make Struct with regret trait
// TODO: Make Struct with strategy updating rule
// TODO: Calculate Exploitability
// TODO: Plot live exploitability in ratatui
// TODO: Stop iteration when exploitability sufficiently small
