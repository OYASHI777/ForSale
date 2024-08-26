use crate::engines::traits::PlayerController;
use crate::models::enums::Player;
use crate::models::game_state::GameState;
use log::debug;
use rand::rngs::ThreadRng;
use rand::seq::IndexedRandom;
use rand::thread_rng;

pub struct MaxNPlayer {
    id: u8,
    nickname: String,
    rng: ThreadRng,
    buffer: Vec<(GameState, u8)>,
}

impl PlayerController for MaxNPlayer {
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
        *legal_moves.choose(&mut self.rng).unwrap()
    }
}

impl MaxNPlayer {
    pub fn new(id: u8, nickname: String) -> Self {
        let rng = thread_rng();
        let buffer: Vec<(GameState, u8)> = Vec::with_capacity(50000);
        MaxNPlayer {
            id,
            nickname,
            rng,
            buffer,
        }
    }
    pub fn score_function(&self, game_state: &GameState) -> Vec<f32> {
        vec![1.0; 6]
    }
    pub fn maximax_depth(&mut self, initial_state: &GameState, turns: u32) -> u8 {
        // TODO: Check to ensure not going into Sell round
        debug!("Initial_state: {}", initial_state);
        let initial_turn_no = initial_state.turn_no();
        let current_player: Player = initial_state.current_player();
        let mut actions: Vec<u8> = initial_state.legal_moves(current_player);
        for action in actions.iter().rev().skip(0) {
            // TODO: The initial state can sometimes be terminal right, to include it below
            let game_state = initial_state.generate_next_state_bid(current_player, *action);
            self.buffer.push((game_state, *action));
        }
        // Gamestate, scores, path_from_initial
        let mut leaf_nodes_buffer: Vec<(GameState, u8, Vec<f32>)> = Vec::with_capacity(14);
        let print_buffer: Vec<u32> = self.buffer.iter().map(|(a, _)| a.turn_no()).collect();
        let print_leaf: Vec<u32> = leaf_nodes_buffer
            .iter()
            .map(|(a, _, _)| a.turn_no())
            .collect();
        debug!("Starting buffer: {:?}", print_buffer);
        debug!("Starting leaf_nodes_buffer: {:?}", print_leaf);
        while self.buffer.len() > 0 {
            let game_state = self.buffer.last().unwrap().clone();
            if game_state.0.turn_no() == initial_turn_no + turns {
                // Terminal Node => get score
                // TODO: one state being terminal doesnt mean all states will be terminal
                let turn_no = self.buffer.last().unwrap().0.turn_no();
                loop {
                    let mut bool_pop = false;
                    if let state_turn_no = self.buffer.last().unwrap().0.turn_no() {
                        bool_pop = state_turn_no == turn_no;
                    }
                    if bool_pop {
                        let pushed_state = self.buffer.pop().unwrap();
                        let score = self.score_function(&pushed_state.0);
                        leaf_nodes_buffer.push((pushed_state.0, pushed_state.1, score));
                    } else {
                        break;
                    }
                }
                let print_buffer: Vec<u32> = self.buffer.iter().map(|(a, _)| a.turn_no()).collect();
                let print_leaf: Vec<u32> = leaf_nodes_buffer
                    .iter()
                    .map(|(a, _, _)| a.turn_no())
                    .collect();
                debug!("Terminal buffer: {:?}", print_buffer);
                debug!("Terminal leaf_nodes_buffer: {:?}", print_leaf);
            } else if leaf_nodes_buffer.len() > 0
                && leaf_nodes_buffer.last().unwrap().0.turn_no()
                    > self.buffer.last().unwrap().0.turn_no()
            {
                // Since leaf buffer contains nodes deeper than the buffer, the last buffer node is the parent of the leaf node
                // Propogate Score
                let mut parent_node = self.buffer.pop().unwrap();
                let current_player = parent_node.0.current_player();
                let mut parent_scores: Vec<f32> = vec![f32::MIN; 6];
                // Calculate score for parent node
                while leaf_nodes_buffer.len() > 0
                    && leaf_nodes_buffer.last().unwrap().0.turn_no() > parent_node.0.turn_no()
                {
                    let child_node: (GameState, u8, Vec<f32>) = leaf_nodes_buffer.pop().unwrap();
                    if child_node.2[current_player as usize]
                        > parent_scores[current_player as usize]
                    {
                        parent_scores = child_node.2;
                    }
                }
                leaf_nodes_buffer.push((parent_node.0, parent_node.1, parent_scores));
                let print_buffer: Vec<u32> = self.buffer.iter().map(|(a, _)| a.turn_no()).collect();
                let print_leaf: Vec<u32> = leaf_nodes_buffer
                    .iter()
                    .map(|(a, _, _)| a.turn_no())
                    .collect();
                debug!("Propagate buffer: {:?}", print_buffer);
                debug!("Propagate leaf_nodes_buffer: {:?}", print_leaf);
            } else {
                // Going deeper into the tree
                let mut actions: Vec<u8> = game_state.0.legal_moves(game_state.0.current_player());
                for action in actions.iter().rev().skip(0) {
                    // TODO: treat differently if its terminal state
                    let next_game_state = game_state
                        .0
                        .generate_next_state_bid(game_state.0.current_player(), *action);
                    self.buffer.push((next_game_state, *action));
                }
                let print_buffer: Vec<u32> = self.buffer.iter().map(|(a, _)| a.turn_no()).collect();
                let print_leaf: Vec<u32> = leaf_nodes_buffer
                    .iter()
                    .map(|(a, _, _)| a.turn_no())
                    .collect();
                debug!("Deepening buffer: {:?}", print_buffer);
                debug!("Deepening leaf_nodes_buffer: {:?}", print_leaf);
            }
        }
        // Return move with Highest Score
        let print_leaf: Vec<u32> = leaf_nodes_buffer
            .iter()
            .map(|(a, _, _)| a.turn_no())
            .collect();
        debug!("Returned leaf_nodes_buffer: {:?}", print_leaf);
        leaf_nodes_buffer
            .iter()
            .max_by(|a, b| {
                a.2[initial_state.current_player() as usize]
                    .partial_cmp(&b.2[initial_state.current_player() as usize])
                    .unwrap()
            })
            .map(|(_, action, _)| *action)
            .unwrap()
    }
}
