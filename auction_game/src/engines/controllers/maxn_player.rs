use crate::engines::controllers::constants::VALUE_PER_PROPERTY;
use crate::engines::traits::PlayerController;
use crate::models::enums::{GamePhase, Player, Property};
use crate::models::game_state::GameState;
use ahash::AHashMap;
use log::{debug, info, warn};
use rand::rngs::ThreadRng;
use rand::seq::IndexedRandom;
use rand::thread_rng;
use std::time::Instant;

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
    pub fn maximax_round(
        &self,
        initial_state: &GameState,
        rounds: u8,
        random_sample: bool,
        n_samples: u32,
    ) -> u8 {
        // TODO: Proof read | Abstract
        // TODO: Test LRUCaching after concurrency
        // TODO: Tune appropriate round depth for optimal plays to 2nd round for a few different start types...
        // TODO: use the self.buffer
        // TODO: Validate the averaging for later rounds
        let start = Instant::now();
        let terminal_round: u8 = initial_state.round_no() + rounds;
        let mut leaf_node_count: u64 = 0;
        let initial_path_encoding = initial_state.get_path_encoding();
        // GameState encoding, Player Scores, number of child nodes remaining, average count
        let mut scores: AHashMap<String, (GameState, Vec<f32>, usize, usize)> =
            AHashMap::with_capacity(100000);
        let mut buffer: Vec<GameState> = Vec::with_capacity(100000);
        let legal_moves = initial_state.legal_moves(initial_state.current_player());
        for action in &legal_moves {
            buffer
                .push(initial_state.manual_next_state_bid(initial_state.current_player(), *action));
        }
        // info!(
        //     "Looking through legal moves: {:?}",
        //     initial_state.legal_moves(initial_state.current_player())
        // );
        scores.insert(
            initial_state.get_path_encoding(),
            (
                initial_state.clone(),
                vec![f32::MIN; initial_state.no_players() as usize],
                legal_moves.len(),
                0,
            ),
        );
        while buffer.len() > 0 {
            let mut leaf_state = buffer.pop().unwrap();
            if leaf_state.auction_end() {
                if leaf_state.round_no() == terminal_round
                    || leaf_state.game_phase() == GamePhase::Sell
                {
                    // Terminal node, return score
                    leaf_node_count += 1;
                    if leaf_node_count % 500000 == 0 {
                        println!(
                            "Visited leaf_nodes: {}, buffer len: {}",
                            leaf_node_count,
                            buffer.len()
                        );
                    }
                    //     TODO: Abstract score function out later on
                    let mut score = MaxNPlayer::round_score_function(&leaf_state);
                    let mut parent_hash = leaf_state.get_parent_encoding();
                    let mut update_parent_further = true;
                    let mut remove_from_scores = false;
                    // TODO: Review if this is most elegant |Fix for issue0
                    if leaf_state.turn_no() == initial_state.turn_no() + 1 {
                        scores.insert(
                            leaf_state.get_path_encoding(),
                            (
                                leaf_state.clone(),
                                Self::round_score_function(&leaf_state),
                                0,
                                0,
                            ),
                        );
                    }

                    // Recursively update the score and remove child score
                    while update_parent_further {
                        if let Some((
                            parent_state,
                            parent_score,
                            remaining_children,
                            mut average_count,
                        )) = scores.get_mut(&parent_hash)
                        {
                            // debug!("Found parent_state: {}", parent_hash,);
                            if parent_state.auction_end() {
                                // Averaging at chance node where new auction is randomly revealed
                                info!("AVERAGING");
                                for player in 0..parent_score.len() {
                                    parent_score[player] = (parent_score[player]
                                        * average_count as f32
                                        + score[player])
                                        / (average_count + 1) as f32;
                                }
                                average_count += 1;
                            } else {
                                // Maximax at deterministic node
                                if parent_score[parent_state.current_player() as usize]
                                    < score[parent_state.current_player() as usize]
                                {
                                    // Update parent score with child score
                                    *parent_score = score.clone();
                                }
                                *remaining_children -= 1;
                                if *remaining_children < 1 {
                                    // Bool to handle removing score in code below
                                    // TODO: Trying this
                                    score = parent_score.clone();
                                    remove_from_scores = true
                                        && parent_state.turn_no() > initial_state.turn_no() + 2;
                                    // && parent_state.turn_no() != initial_state.turn_no() + 1;
                                } else {
                                    update_parent_further = false;
                                }
                            }
                        } else {
                            debug_assert!(false, "Should never reach here. scores should always have a parent_hash for DFS");
                            update_parent_further = false;
                        }
                        debug!("PROPAGATING: Old parent state: {}", parent_hash);
                        if parent_hash == initial_path_encoding {
                            // Stop propagating deletions
                            break;
                        }
                        // TODO: dont really need this part if update_parent_further is false...
                        if update_parent_further {
                            if remove_from_scores {
                                leaf_state = scores.remove(&parent_hash).unwrap().0;
                                parent_hash = leaf_state.get_parent_encoding();
                                remove_from_scores = false;
                            } else {
                                if let Some((state, _, _, _)) = scores.get(&parent_hash) {
                                    debug!(
                                        "Trying to get parent hash of the current node: {}",
                                        parent_hash
                                    );
                                    parent_hash = state.get_parent_encoding();
                                } else {
                                    warn!(
                                        "Unable to find the parent hash: {} in scores",
                                        parent_hash
                                    );
                                    panic!();
                                }
                            }
                        } else {
                            if remove_from_scores {
                                scores.remove(&parent_hash);
                            }
                        }
                    }
                } else {
                    // Auction end but not terminal node => try every combo and average score
                    // This part is not computationally feasible LOL
                    let chances_leaves = leaf_state.reveal_auction_perms(random_sample, n_samples);
                    scores.insert(
                        leaf_state.get_path_encoding(),
                        (
                            leaf_state.clone(),
                            vec![f32::MIN; leaf_state.no_players() as usize],
                            chances_leaves.len(),
                            0,
                        ),
                    );
                    buffer.extend(chances_leaves);
                }
            } else {
                // Deepening and adding child nodes to buffer
                // TODO: Abstract this
                let legal_moves = leaf_state.legal_moves(leaf_state.current_player());
                let child_states_count: usize = legal_moves.len();
                for action in legal_moves {
                    let child_state =
                        leaf_state.manual_next_state_bid(leaf_state.current_player(), action);
                    buffer.push(child_state);
                }
                scores.insert(
                    leaf_state.get_path_encoding(),
                    (
                        leaf_state.clone(),
                        vec![f32::MIN; leaf_state.no_players() as usize],
                        child_states_count,
                        0,
                    ),
                );
            }
        }
        // TODO: Figure a way to store and display this
        // TODO: Store the best action
        let mut best_action: u8 = 0;
        let mut best_score: f32 = f32::MIN;
        for action in initial_state.legal_moves(initial_state.current_player()) {
            let next_state =
                initial_state.manual_next_state_bid(initial_state.current_player(), action);
            if let Some((_, score, _, _)) = scores.get(&next_state.get_path_encoding()) {
                info!(
                    "FINAL: Player : {}, Action: {action} Scores: {:?}",
                    initial_state.current_player(),
                    score
                );
                if best_score < score[initial_state.current_player() as usize] {
                    best_action = action;
                    best_score = score[initial_state.current_player() as usize];
                }
            } else {
                info!(
                    "FINAL: Scores not found for {}",
                    next_state.get_path_encoding()
                );
            }
        }
        let round_2 =
            initial_state.manual_next_state_bid(initial_state.current_player(), best_action);
        if !round_2.auction_end() {
            for action in round_2.legal_moves(round_2.current_player()) {
                let next_state = round_2.manual_next_state_bid(round_2.current_player(), action);
                if let Some((_, score, _, _)) = scores.get(&next_state.get_path_encoding()) {
                    info!(
                        "FINAL (Round 2): Player : {}, Action: {action} Scores: {:?}",
                        round_2.current_player(),
                        score
                    );
                } else {
                    info!(
                        "FINAL: Scores not found for {}",
                        next_state.get_path_encoding()
                    );
                }
            }
        }

        info!("MAXN algo ran for: {:?}", start.elapsed());
        info!("Ended with leaf_nodes count: {}", leaf_node_count);
        best_action
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
        debug_assert!(
            scores.len() == game_state.no_players() as usize,
            "Returning scores :{:?} is not equal to no_players: {}",
            scores,
            game_state.no_players()
        );
        scores
    }
}
