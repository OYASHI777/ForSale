use crate::engines::q_values::regret::Regret;
use crate::engines::scorers::naive_round_score::NaiveRoundScore;
use crate::engines::utils::{mixed_strategy_score, normalize, sample_strategy, update_average};
use crate::game_modes::traits::Game;
use crate::models::enums::{GamePhase, Player};
use crate::models::game_state::GameState;
use ahash::AHashMap;
use bimap::BiMap;
use itertools::Itertools;
use log::{debug, info};
use rand::{thread_rng, Rng};

pub struct CFR<T>
where
    T: Terminator,
{
    move_map: AHashMap<String, Vec<BiMap<usize, u8>>>,
    strategy: AHashMap<String, Vec<Vec<f32>>>, // These are probabilities of taking an action
    regret: AHashMap<String, Vec<Vec<f32>>>,
    value: AHashMap<String, Vec<Vec<f32>>>,
    buffer: Vec<GameState>,
    terminator: T,
    alternating_update: bool,
}

impl<T> CFR<T> {
    pub fn new(terminator: T, alternating_update: bool) -> Self {
        let move_map: AHashMap<String, Vec<BiMap<usize, u8>>> = AHashMap::with_capacity(1);
        let strategy: AHashMap<String, Vec<Vec<f32>>> = AHashMap::with_capacity(1);
        let regret: AHashMap<String, Vec<Vec<f32>>> = AHashMap::with_capacity(1);
        let value: AHashMap<String, Vec<Vec<f32>>> = AHashMap::with_capacity(1);
        let buffer: Vec<GameState> = Vec::with_capacity(1000);
        CFR {
            move_map,
            strategy,
            regret,
            value,
            buffer,
            terminator,
            alternating_update,
        }
    }

    pub fn initialise_node(&mut self, game_state: &GameState) {
        // Creates the strategies, q_values and move_map for each player in the state.
        if game_state.game_phase() == GamePhase::Sell {
            let path = game_state.get_path_encoding();
            let no_players = game_state.no_players();
            let no_moves = game_state.legal_moves(0).len(); // This is a shortcut that works for For Sale
            let initial_strategies: Vec<Vec<f32>> =
                vec![vec![1.0 / no_players as f32; no_moves]; no_players as usize];
            let initial_value: Vec<Vec<f32>> = vec![vec![0.0; no_moves]; no_players as usize];
            self.strategy
                .insert(path.clone(), initial_strategies.clone());
            self.regret.insert(path.clone(), initial_strategies);
            self.value.insert(path.clone(), initial_value);
            let mut move_map_vec: Vec<BiMap<usize, u8>> =
                Vec::with_capacity(game_state.no_players() as usize);
            for player in 0..no_players {
                let player_legal_moves = game_state.legal_moves(player);
                let mut player_move_map: BiMap<usize, u8> = BiMap::with_capacity(10);
                for (index, action) in player_legal_moves.iter().enumerate() {
                    player_move_map.insert(index, *action);
                }
                move_map_vec.push(player_move_map);
            }
            self.move_map.insert(path.clone(), move_map_vec);
        } else {
            todo!("Focused on Greedy Baby for now");
        }
    }

    pub fn add_game_state(&mut self, game_state: GameState) {
        self.buffer.push(game_state);
    }

    pub fn game_state_added(&self, game_state: &GameState) -> bool {
        self.strategy.get(&game_state.get_path_encoding()).is_some()
    }

    pub fn get_mixed_strategy(&self, game_state: &GameState, player: Player) -> u8 {
        let path = game_state.get_path_encoding();
        if let Some(strategies) = self.strategy.get(&path) {
            let index = sample_strategy(&strategies[player as usize]);
            if let Some(move_maps) = self.move_map.get(&path) {
                if let Some(action) = move_maps[player as usize].get_by_left(&index) {
                    *action
                } else {
                    panic!("Failed to find appropriate action");
                }
            } else {
                panic!("Failed to find move_map for player: {}", player);
            }
        } else {
            panic!("Failed to find strategy!");
        }
    }

    pub fn find_nash(&mut self, initial_state: &GameState, iterations: usize) {
        // Proper way is to simulate every outcome

        // For each player, get regret
        //      For each legal move
        //      Simulate all other moves based on strategy
        //      Update regret
        // For all q_values update the strategy
        let path = initial_state.get_path_encoding();
        if !self.strategy.contains(&path) {
            self.initialise_node(&initial_state);
        }
        for i in 0..iterations {
            self.cfr(&initial_state, i);
            if i % 100 == 0 {
                // println!("STRATEGY: ITER: {}", i);
                // for player in 0..initial_state.no_players() as usize {
                //     println!("P{}: {:?}", player, strategy_vec[player]);
                // }
                // println!("Q VALUE: ITER: {}", i);
                // for player in 0..initial_state.no_players() as usize {
                //     println!("P{}: {:?}", player, value_vec[player]);
                // }
                // let mut player_values: Vec<f32> = Vec::with_capacity(6);
                // for player in 0..initial_state.no_players() as usize {
                //     let mixed_policy_score =
                //         mixed_strategy_score(&strategy_vec[player], &value_vec[player]);
                //     player_values.push(mixed_policy_score);
                // }
                // println!("{:?}", player_values);

                // ====== EXPLOITABILITY ======
                // let mut total_exploitability: f32 = 0.0;
                // for player in 0..initial_state.no_players() as usize {
                //     let mixed_policy_score =
                //         mixed_strategy_score(&strategy_vec[player], &value_vec[player]);
                //     let max_q_value = value_vec[player]
                //         .iter()
                //         .max_by(|&a, &b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal))
                //         .cloned()
                //         .unwrap_or(0.0);
                //     total_exploitability += max_q_value - mixed_policy_score;
                // }
                // println!("ITER: {} % EXPLOITABILITY: {}", i, total_exploitability);
            }
        }
        let strategy_vec = match self.strategy.get_mut(&path) {
            Some(strategy_vec) => strategy_vec,
            None => panic!("Failed to find appropriate strategy"),
        };
        // let regret_vec = match self.regret.get_mut(&path) {
        //     Some(regret_vec) => regret_vec,
        //     None => panic!("Failed to find appropriate q_value"),
        // };
        // let value_vec = match self.value.get_mut(&path) {
        //     Some(value_vec) => value_vec,
        //     None => panic!("Failed to find appropriate q_value"),
        // };
        // let move_map = match self.move_map.get(&path) {
        //     Some(move_map) => move_map,
        //     None => panic!("Failed to find appropriate move_map"),
        // };
        info!("PLAYER STRATEGY");
        for player in 0..initial_state.no_players() as usize {
            info!("P{}: {:?}", player, strategy_vec[player]);
        }
    }

    fn cfr(&mut self, initial_state: &&GameState, i: usize) -> Vec<f32> {
        if self.terminator.is_terminal() {
            return NaiveRoundScore::round_score_function(&initial_state);
        }
        let path = initial_state.get_path_encoding();
        if !self.strategy.contains(&path) {
            self.initialise_node(&initial_state);
        }
        let strategy_vec = match self.strategy.get_mut(&path) {
            Some(strategy_vec) => strategy_vec,
            None => panic!("Failed to find appropriate strategy"),
        };
        let regret_vec = match self.regret.get_mut(&path) {
            Some(regret_vec) => regret_vec,
            None => panic!("Failed to find appropriate q_value"),
        };
        let value_vec = match self.value.get_mut(&path) {
            Some(value_vec) => value_vec,
            None => panic!("Failed to find appropriate q_value"),
        };
        let move_map = match self.move_map.get(&path) {
            Some(move_map) => move_map,
            None => panic!("Failed to find appropriate move_map"),
        };
        let mut returned_scores: Vec<Vec<f32>> = vec![
            vec![0.0; initial_state.no_players() as usize];
            initial_state.no_players() as usize
        ];
        for update_player in 0..initial_state.no_players() as usize {
            let legal_moves = &initial_state.legal_moves(update_player as u8);
            // let mut temp_scores: Vec<f32> = vec![0.0; legal_moves.len()];
            let mut temp_scores: &mut Vec<f32> = returned_scores.get_mut(update_player).unwrap();
            for move_index in 0..legal_moves.len() {
                let mut aggregate_sales: Vec<u8> =
                    Vec::with_capacity(initial_state.no_players() as usize);
                let action = legal_moves[move_index];
                for move_player in 0..initial_state.no_players() {
                    if move_player == update_player as u8 {
                        aggregate_sales.push(action);
                    } else {
                        let sampled_strategy_index =
                            sample_strategy(&strategy_vec[move_player as usize]);
                        let sampled_action: u8 = match move_map[move_player as usize]
                            .get_by_left(&sampled_strategy_index)
                        {
                            Some(action) => *action,
                            None => {
                                panic!("Failed to find appropriate action in move_map");
                            }
                        };
                        aggregate_sales.push(sampled_action);
                    }
                }
                // Evaluate and update q_value based on action
                // TODO: The random choice is not working
                let game_state = initial_state.generate_next_state_sell(aggregate_sales.clone());
                let score: f32 = NaiveRoundScore::round_score_function(&game_state)[update_player];
                temp_scores[move_index] = score;
            }
            // TODO: Store iterations with strategy, regret, q_value so they can be updated within
            // TODO: Create a struct for this
            //     Regret matching
            //     Get average utility
            let average_score = mixed_strategy_score(&strategy_vec[update_player], &temp_scores);
            temp_scores
                .iter_mut()
                .for_each(|s| *s = (*s - average_score).max(0.0));
            // Calculating Regret
            for (q, t) in regret_vec[update_player].iter_mut().zip(temp_scores.iter()) {
                // CFR
                // *q += t;
                // CFR+
                *q = (*q + t).max(0.0);
            }
            if self.alternating_update {
                normalize(&mut strategy_vec[update_player], &regret_vec[update_player]);
            }
            //     Update strategy
            update_average(&mut value_vec[update_player], &temp_scores, i + 1);
        }
        if !self.alternating_update {
            for update_player in 0..initial_state.no_players() as usize {
                normalize(&mut strategy_vec[update_player], &regret_vec[update_player]);
            }
        }
    }
}

struct TerminalNode {
    initial_node: GameState,
    terminal_round: Option<u8>,
    terminal_turn: Option<u32>,
}

impl TerminalNode {
    pub fn turn_terminator(initial_node: &GameState, no_search_turns: u32) -> Self {
        TerminalNode {
            initial_node: initial_node.clone(),
            terminal_round: None,
            terminal_turn: Some(initial_node.turn_no() + no_search_turns),
        }
    }
    pub fn round_terminator(initial_node: &GameState, no_search_rounds: u8) -> Self {
        TerminalNode {
            initial_node: initial_node.clone(),
            terminal_round: Some(initial_node.round_no() + no_search_rounds),
            terminal_turn: None,
        }
    }
}

trait Terminator {
    fn is_terminal(&self, node: &GameState) -> bool;
}

impl Terminator for TerminalNode {
    fn is_terminal(&self, node: &GameState) -> bool {
        if let Some(value) = self.terminal_round {
            node.round_no() >= value
        } else if let Some(value) = self.terminal_turn {
            node.turn_no() >= value
        } else {
            panic!("You really should not be here")
        }
    }
}

// TODO: Make Struct with regret trait
// TODO: Make Struct with strategy updating rule
// TODO: Calculate Exploitability
// TODO: Plot live exploitability in ratatui
// TODO: Stop iteration when exploitability sufficiently small
// https://arxiv.org/pdf/1407.5042
// https://openreview.net/pdf?id=rJx4p3NYDB
// ESCHER
