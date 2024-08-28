use crate::engines::controllers::constants::VALUE_PER_PROPERTY;
use crate::engines::traits::PlayerController;
use crate::models::enums::{GamePhase, Player, Property};
use crate::models::game_state::GameState;
use ahash::AHashMap;
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
        let mut leaf_nodes_buffer: Vec<(GameState, u8, Vec<f32>)> = Vec::with_capacity(28);
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
    pub fn maximax_round(
        &mut self,
        initial_state: &GameState,
        rounds: u8,
        random_sample: bool,
        n_samples: u32,
    ) -> u8 {
        let parent_score_map: AHashMap<GameState, (u64, Vec<f32>)> =
            AHashMap::with_capacity(100000);
        // Score: parent_hash, node's gamestate
        // GameState hash, Player Scores, number of child nodes remaining
        let terminal_round: u8 = initial_state.round_no() + rounds;

        let mut scores: AHashMap<u64, (GameState, Vec<f32>, usize)> =
            AHashMap::with_capacity(100000);
        let mut buffer: Vec<GameState> = Vec::with_capacity(100000);
        buffer.push(initial_state.clone());
        while buffer.len() > 0 {
            let mut leaf_state = buffer.pop().unwrap();
            if leaf_state.auction_end() {
                if leaf_state.round_no() == terminal_round
                    || leaf_state.game_phase() == GamePhase::Sell
                {
                    // Terminal node, return score
                    //     TODO: Abstract score function out later on
                    let mut score = MaxNPlayer::round_score_function(&leaf_state);
                    let mut child_hash: u64 = leaf_state.get_hash();
                    let mut parent_hash = leaf_state.get_parent_hash();
                    let parent_player = leaf_state.previous_player();
                    let mut update_parent_further = true;
                    let mut remove_from_scores = false;
                    // Recursively update the score and remove child score
                    // TODO: Keep child scores for auction_end() rounds only
                    // TODO: When updating auction_end() scores that are not terminal, to use average (may need to count how many so far)
                    while update_parent_further {
                        if let Some((_parent_state, parent_score, remaining_children)) =
                            scores.get_mut(&parent_hash)
                        {
                            let child_score = score.clone();
                            if parent_score[parent_player as usize] < score[parent_player as usize]
                            {
                                // Update parent score
                                *parent_score = child_score;
                                *remaining_children -= 1;
                                if *remaining_children < 1 {
                                    remove_from_scores = true;
                                } else {
                                    update_parent_further = false;
                                }
                            }
                        }
                        {
                            debug_assert!(false, "Should never reach here. scores should always have a parent_hash for DFS");
                            update_parent_further = false;
                        }
                        if remove_from_scores {
                            // parent state now is assigned to leaf state
                            leaf_state = scores.remove(&parent_hash).unwrap().0;
                            parent_hash = leaf_state.get_parent_hash();
                            remove_from_scores = false;
                        }
                    }
                } else {
                    // Auction end but not terminal node => try every combo and average score
                    let chances_leaves = leaf_state.reveal_auction_perms(random_sample, n_samples);
                    scores.insert(
                        leaf_state.get_hash(),
                        (
                            leaf_state.clone(),
                            vec![f32::MIN; leaf_state.no_players() as usize],
                            chances_leaves.len(),
                        ),
                    );
                    buffer.extend(chances_leaves);
                }
            } else {
                // Deepening and adding child nodes to buffer
                // TODO: Abstract this
                let legal_moves = leaf_state.legal_moves(leaf_state.current_player());
                let mut child_states: Vec<GameState> = Vec::with_capacity(28);
                for action in legal_moves {
                    let child_state =
                        leaf_state.manual_next_state_bid(leaf_state.current_player(), action);
                    child_states.push(child_state);
                }
                scores.insert(
                    leaf_state.get_hash(),
                    (
                        leaf_state.clone(),
                        vec![f32::MIN; leaf_state.no_players() as usize],
                        child_states.len(),
                    ),
                );
                buffer.extend(child_states);
            }
        }
        todo!()
    }
    fn round_score_function(game_state: &GameState) -> Vec<f32> {
        debug_assert!(
            game_state.auction_end() == true,
            "Cannot use round_score_function if round has not ended!"
        );
        // TODO: Expand beyond 6 players at some point
        let mut scores: Vec<f32> = vec![0.0; game_state.no_players() as usize];
        match game_state.game_phase() {
            GamePhase::Bid => {
                // For each property multiply by point
                // Calculate the remaining properties/ remaining coins
                // for each coin multiply by the ratio
                let coins = game_state.get_coins();
                let remaining_property_per_coin: f32 = game_state
                    .get_remaining_properties()
                    .iter()
                    .map(|&prop| prop as f32)
                    .sum::<f32>()
                    / coins.iter().map(|&prop| prop as f32).sum::<f32>();
                for i in 0..game_state.no_players() {
                    let player_properties = game_state.get_player_properties(i);
                    scores[i as usize] += VALUE_PER_PROPERTY
                        * player_properties
                            .iter()
                            .map(|&prop| prop as f32)
                            .sum::<f32>()
                        + VALUE_PER_PROPERTY
                            * remaining_property_per_coin
                            * coins[i as usize] as f32;
                }
            }
            GamePhase::Sell => {
                let properties: &AHashMap<Player, Vec<Property>> = game_state.get_properties();
                let total_remaining_properties: f32 = properties
                    .values()
                    .flat_map(|props| props.iter())
                    .map(|&prop| prop as f32)
                    .sum::<f32>()
                    .into();
                let remaining_checks_per_property: f32 = game_state
                    .get_remaining_checks()
                    .iter()
                    .map(|&prop| prop as f32)
                    .sum::<f32>()
                    / total_remaining_properties;
                for i in 0..game_state.no_players() {
                    let player_checks = game_state.get_player_checks(i);
                    let player_properties = game_state.get_player_properties(i);
                    scores[i as usize] +=
                        player_checks.iter().map(|&prop| prop as f32).sum::<f32>()
                            + remaining_checks_per_property
                                * player_properties
                                    .iter()
                                    .map(|&prop| prop as f32)
                                    .sum::<f32>();
                }
            }
        }
        scores
    }
}
