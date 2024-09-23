use crate::engines::constants::VALUE_PER_PROPERTY;
use crate::engines::traits::PlayerController;
use crate::models::enums::{GamePhase, Player, Property};
use crate::models::game_state::GameState;
use ahash::AHashMap;
use log::{debug, info, warn};
use num_traits::float::FloatCore;
use rand::rngs::ThreadRng;
use rand::seq::IndexedRandom;
use rand::thread_rng;
use std::cmp;
use std::time::Instant;

pub struct MaxNPlayer {
    id: u8,
    nickname: String,
    rng: ThreadRng,
    buffer: Vec<GameState>,
    // GameState encoding, Player Scores, number of child nodes remaining, average count
    scores: AHashMap<String, (GameState, Vec<f32>, usize, usize)>,
    bool_print: bool,
    bool_log: bool,
}

impl PlayerController for MaxNPlayer {
    fn nickname(&self) -> String {
        self.nickname.clone()
    }
    fn decision(&mut self, game_state: &GameState) -> u8 {
        // TODO: Might consider making these stored params
        let best_move = self.maximax_round(&game_state, 1, false, 0);
        best_move
    }
    fn batch_decision(&mut self, game_state: &GameState) -> Vec<u8> {
        todo!()
        //     TODO: Remove controller trait for engines, move engines elsewhere and have controllers run engines
        //     TODO: So maxn should be an engine not a controller
    }
}

impl MaxNPlayer {
    pub fn new(id: u8, nickname: String, bool_print: bool, bool_log: bool) -> Self {
        let rng = thread_rng();
        let buffer: Vec<GameState> = Vec::with_capacity(10000);
        let scores: AHashMap<String, (GameState, Vec<f32>, usize, usize)> =
            AHashMap::with_capacity(30000);
        MaxNPlayer {
            id,
            nickname,
            rng,
            buffer,
            scores,
            bool_print,
            bool_log,
        }
    }
    pub fn maximax_round(
        &mut self,
        initial_state: &GameState,
        rounds: u8,
        random_sample: bool,
        n_samples: u32,
    ) -> u8 {
        // TODO: Test LRUCaching after concurrency
        let start = Instant::now();
        let terminal_round: u8 = initial_state.round_no() + rounds;
        let mut leaf_node_count: u64 = 0;
        let initial_path_encoding = initial_state.get_path_encoding();
        let legal_moves = initial_state.legal_moves(initial_state.current_player());
        for action in &legal_moves {
            self.buffer
                .push(initial_state.manual_next_state_bid(initial_state.current_player(), *action));
        }
        self.scores.insert(
            initial_state.get_path_encoding(),
            (
                initial_state.clone(),
                vec![f32::MIN; initial_state.no_players() as usize],
                legal_moves.len(),
                0,
            ),
        );
        while self.buffer.len() > 0 {
            let mut leaf_state = self.buffer.pop().unwrap();
            if leaf_state.auction_end() {
                if leaf_state.round_no() == terminal_round
                    || leaf_state.game_phase() == GamePhase::Sell
                {
                    // Terminal node, return score
                    leaf_node_count += 1;
                    if self.bool_print && leaf_node_count % 10000000 == 0 {
                        println!(
                            "Visited leaf_nodes: {}, buffer len: {}",
                            leaf_node_count,
                            self.buffer.len()
                        );
                    }
                    // Propagate Scores
                    self.update_score(initial_state, &initial_path_encoding, &mut leaf_state);
                } else {
                    // Auction end but not terminal node => try every combo and average score
                    let chances_leaves = leaf_state.reveal_auction_perms(random_sample, n_samples);
                    self.scores.insert(
                        leaf_state.get_path_encoding(),
                        (
                            leaf_state.clone(),
                            vec![f32::MIN; leaf_state.no_players() as usize],
                            chances_leaves.len(),
                            0,
                        ),
                    );
                    self.buffer.extend(chances_leaves);
                }
            } else {
                // Deepening and adding child nodes to buffer
                self.deepen_search(leaf_state);
            }
        }
        self.get_best_action(initial_state, start, &leaf_node_count)
    }

    fn deepen_search(&mut self, mut leaf_state: GameState) {
        let legal_moves = leaf_state.legal_moves(leaf_state.current_player());
        let child_states_count: usize = legal_moves.len();
        for action in legal_moves {
            let child_state = leaf_state.manual_next_state_bid(leaf_state.current_player(), action);
            self.buffer.push(child_state);
        }
        self.scores.insert(
            leaf_state.get_path_encoding(),
            (
                leaf_state.clone(),
                vec![f32::MIN; leaf_state.no_players() as usize],
                child_states_count,
                0,
            ),
        );
    }

    fn get_best_action(
        &mut self,
        initial_state: &GameState,
        start: Instant,
        leaf_node_count: &u64,
    ) -> u8 {
        let mut best_action: u8 = 0;
        let mut best_score: f32 = f32::MIN;
        for action in initial_state.legal_moves(initial_state.current_player()) {
            let next_state =
                initial_state.manual_next_state_bid(initial_state.current_player(), action);
            if let Some((_, score, _, _)) = self.scores.get(&next_state.get_path_encoding()) {
                if self.bool_log {
                    info!(
                        "FINAL: Player : {}, Action: {action} Scores: {:?}",
                        initial_state.current_player() + 1,
                        score
                    );
                }
                if best_score < score[initial_state.current_player() as usize] {
                    best_action = action;
                    best_score = score[initial_state.current_player() as usize];
                }
            } else {
                if self.bool_log {
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

    fn update_score(
        &mut self,
        initial_state: &GameState,
        initial_path_encoding: &str,
        mut leaf_state: &mut GameState,
    ) {
        let mut score = MaxNPlayer::round_score_function(&leaf_state);
        let mut parent_hash = leaf_state.get_parent_encoding();
        let mut update_parent_further = true;
        let mut remove_from_scores = false;
        // TODO: Review if this is most elegant |Fix for issue0
        if leaf_state.turn_no() == initial_state.turn_no() + 1 {
            self.scores.insert(
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
            if let Some((parent_state, parent_score, remaining_children, mut average_count)) =
                self.scores.get_mut(&parent_hash)
            {
                if parent_state.auction_end() {
                    // Averaging at chance node where new auction is randomly revealed
                    for player in 0..parent_score.len() {
                        parent_score[player] = (parent_score[player] * average_count as f32
                            + score[player])
                            / (average_count + 1) as f32;
                    }
                    average_count += 1;
                    //     TODO: Make average node propagate upwards...
                    *remaining_children -= 1;
                    if *remaining_children < 1 {
                        // Bool to handle removing score in code below
                        // TODO: Trying this
                        score = parent_score.clone();
                        remove_from_scores =
                            true && parent_state.turn_no() > initial_state.turn_no() + 2;
                        // && parent_state.turn_no() != initial_state.turn_no() + 1;
                    } else {
                        update_parent_further = false;
                    }
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
                        // THIS IS WHERE YOU DECIDE HOW MUCH SCORES TO SAVE
                        remove_from_scores =
                            true && parent_state.turn_no() > initial_state.turn_no() + 2;
                        // && parent_state.turn_no() != initial_state.turn_no() + 1;
                    } else {
                        update_parent_further = false;
                    }
                }
            } else {
                debug_assert!(
                    false,
                    "Should never reach here. scores should always have a parent_hash for DFS"
                );
                update_parent_further = false;
            }
            if parent_hash == initial_path_encoding.to_string() {
                // Stop propagating deletions
                break;
            }
            // TODO: dont really need this part if update_parent_further is false...
            if update_parent_further {
                if remove_from_scores {
                    *leaf_state = self.scores.remove(&parent_hash).unwrap().0;
                    parent_hash = leaf_state.get_parent_encoding();
                    remove_from_scores = false;
                } else {
                    if let Some((state, _, _, _)) = self.scores.get(&parent_hash) {
                        parent_hash = state.get_parent_encoding();
                    } else {
                        warn!("Unable to find the parent hash: {} in scores", parent_hash);
                        panic!();
                    }
                }
            } else {
                if remove_from_scores {
                    self.scores.remove(&parent_hash);
                }
            }
        }
    }

    pub fn round_score_function(game_state: &GameState) -> Vec<f32> {
        debug_assert!(
            game_state.auction_end() == true,
            "Cannot use round_score_function if round has not ended!"
        );
        // TODO: Expand beyond 6 players at some point
        let mut scores: Vec<f32> = vec![0.0; game_state.no_players() as usize];
        // let bool_bid_phase_score_function: bool = game_state.game_phase() == GamePhase::Bid
        //     || game_state.get_remaining_checks().len() > 24;
        let coins = game_state.get_coins();
        if game_state.game_end() {
            let mut total_score: f32 = 0.0;
            for i in 0..game_state.no_players() {
                let player_checks = game_state.get_player_checks(i);
                scores[i as usize] += player_checks.iter().map(|&prop| prop as f32).sum::<f32>()
                    + coins[i as usize] as f32;
                total_score += scores[i as usize]
            }
            for score in scores.iter_mut() {
                *score /= total_score;
            }
            return scores;
        }
        let mut max_score: f32 = f32::MIN;
        let mut total_score: f32 = 0.0;
        match game_state.game_phase() {
            GamePhase::Bid => {
                // For each property multiply by point
                // Calculate the remaining properties/ remaining coins
                // for each coin multiply by the ratio
                let total_coins = coins.iter().map(|&prop| prop as f32).sum::<f32>();
                let remaining_property_per_coin: f32 = if total_coins == 0.0 {
                    0.0
                } else {
                    game_state
                        .get_remaining_properties()
                        .iter()
                        .map(|&prop| prop as f32)
                        .sum::<f32>()
                        / total_coins
                };
                let value_per_coin: f32 =
                    (VALUE_PER_PROPERTY * remaining_property_per_coin).max(1.0);

                for i in 0..game_state.no_players() {
                    let player_properties = game_state.get_player_properties(i);
                    scores[i as usize] += VALUE_PER_PROPERTY
                        * player_properties
                            .iter()
                            .map(|&prop| prop as f32)
                            .sum::<f32>()
                        + value_per_coin * coins[i as usize] as f32;
                    total_score += scores[i as usize];
                    if scores[i as usize] > max_score {
                        max_score = scores[i as usize];
                    }
                }
            }
            GamePhase::Sell => {
                let properties: &AHashMap<Player, Vec<Property>> = game_state.get_properties();
                // TODO: Handle case where game has ended...
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
                                    .sum::<f32>()
                            + coins[i as usize] as f32;
                    total_score += scores[i as usize];
                    if scores[i as usize] > max_score {
                        max_score = scores[i as usize];
                    }
                }
            }
        }
        for score in scores.iter_mut() {
            *score = (*score - max_score) / total_score;
        }
        let total_advantage: f32 = -scores.iter().sum::<f32>();
        for score in scores.iter_mut() {
            if (*score - 0.0).abs() < f32::epsilon() {
                *score += total_advantage;
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
