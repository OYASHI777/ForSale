use crate::models::enums::{Check, Coins, GamePhase, Player, Property};
use ahash::AHashMap;
use rand::seq::SliceRandom;
use rand::thread_rng;
use std::fmt;
use std::fmt::Write;

#[derive(Clone)]
pub struct GameState {
    game_phase: GamePhase,
    no_players: Player,
    coins: Vec<Coins>,
    properties: AHashMap<Player, Vec<Property>>,
    checks: AHashMap<Player, Vec<Check>>,
    current_decision_player: Option<u8>,
    active_players: Vec<bool>,
    active_bids: Vec<u8>,
    auction_pool: Vec<u8>,
    remaining_properties: Vec<Property>,
    remaining_checks: Vec<Check>,
    round_winner: Option<Player>,
    round_no: u32,
}

impl GameState {
    pub fn starting(no_players: u8) -> Self {
        debug_assert!(
            no_players < 7,
            "Please ensure no_players is < 7. It is currently {no_players}"
        );
        debug_assert!(
            no_players > 2,
            "Please ensure no_players is > 2. It is currently {no_players}"
        );

        let starting_coins = match no_players {
            6 => 14,
            5 => 16,
            4 => 21,
            3 => 28,
            _ => {
                panic!("Please ensure 3 <= no_players <= 6. Received no_players = {no_players}")
            }
        };
        let mut coins: Vec<Coins> = Vec::with_capacity(no_players as usize);
        let mut active_players: Vec<bool> = Vec::with_capacity(no_players as usize);
        let active_bids: Vec<u8> = vec![0; 6];
        for _ in 0..no_players {
            coins.push(starting_coins);
            active_players.push(true);
        }
        let mut remaining_properties: Vec<Property> = (1..=30).collect();
        let mut rng = thread_rng();
        remaining_properties.shuffle(&mut rng);
        let mut remaining_checks: Vec<Check> = vec![
            0, 0, 2, 2, 3, 3, 4, 4, 5, 5, 6, 6, 7, 7, 8, 8, 9, 9, 10, 10, 11, 11, 12, 12, 13, 13,
            14, 14, 15, 15,
        ];
        remaining_checks.shuffle(&mut rng);

        let mut properties: AHashMap<Player, Vec<Property>> =
            AHashMap::with_capacity(no_players as usize);
        let mut checks: AHashMap<Player, Vec<Check>> = AHashMap::with_capacity(no_players as usize);
        for i in 0..no_players {
            properties.insert(i, Vec::with_capacity(10));
            checks.insert(i, Vec::with_capacity(10));
        }
        let auction_properties: Vec<u8> = Vec::with_capacity(no_players as usize);
        // TODO: Randomize
        let current_decision_player: Option<u8> = Some(0);
        GameState {
            game_phase: GamePhase::Bid,
            no_players,
            coins,
            properties,
            checks,
            current_decision_player,
            active_players,
            active_bids,
            auction_pool: auction_properties,
            remaining_properties,
            remaining_checks,
            round_winner: None,
            round_no: 0,
        }
    }
    pub fn current_player(&self) -> Player {
        // TODO: Consider changing for phase sell
        self.current_decision_player.unwrap()
    }
    pub fn game_phase(&self) -> GamePhase {
        self.game_phase
    }
    pub fn add_coins(&mut self, player: Player, amount: u8) {
        debug_assert!(
            player < self.no_players,
            "Player number of {player} too high! Keep it less than {}",
            &self.no_players
        );
        self.coins[player as usize] += amount;
    }
    pub fn remove_coins(&mut self, player: Player, amount: u8) {
        debug_assert!(
            player < self.no_players,
            "Player number of {player} too high! Keep it less than {}",
            &self.no_players
        );
        debug_assert!(
            self.coins[player as usize] >= amount,
            "Player {player} only has {} coins. Cannot deduct {amount} from them.",
            &self.coins[player as usize]
        );
        self.coins[player as usize] -= amount;
    }
    fn insert_in_order(vec: &mut Vec<u8>, value: u8) {
        match vec.binary_search(&value) {
            Ok(pos) => vec.insert(pos, value), // Insert at correct position (duplicates allowed)
            Err(pos) => vec.insert(pos, value), // Insert at the correct position (if not found)
        }
    }
    pub fn insert_check_ascending(&mut self, player: Player, check: Check) {
        debug_assert!(
            player < self.no_players,
            "Player number of {player} too high! Keep it less than {}",
            &self.no_players
        );
        if let Some(checks) = self.checks.get_mut(&player) {
            Self::insert_in_order(checks, check);
        } else {
            debug_assert!(false, "Failed to find player {player} in self.checks");
        }
    }
    pub fn insert_property_ascending(&mut self, player: Player, property: Property) {
        debug_assert!(
            player < self.no_players,
            "Player number of {player} too high! Keep it less than {}",
            &self.no_players
        );
        if let Some(properties) = self.properties.get_mut(&player) {
            Self::insert_in_order(properties, property);
        } else {
            debug_assert!(false, "Failed to find player {player} in self.checks");
        }
    }
    pub fn player_now_inactive(&mut self, player: Player) {
        debug_assert!(
            player < self.no_players,
            "Player number of {player} too high! Keep it less than {}",
            &self.no_players
        );
        self.active_players[player as usize] = false;
    }
    pub fn reset_round(&mut self) {
        for bid in self.active_bids.iter_mut() {
            *bid = 0;
        }
        for activity in self.active_players.iter_mut() {
            *activity = true;
        }
        self.current_decision_player = self.round_winner;
    }
    pub fn bid_round_end(&self) -> bool {
        if self.auction_properties_remaining() == 0 {
            true
        } else {
            false
        }
    }
    pub fn auction_properties_remaining(&self) -> u8 {
        self.auction_pool.len() as u8
    }
    pub fn next_player_bid(&self) -> Player {
        debug_assert!(
            self.game_phase == GamePhase::Bid,
            "next_player_bid only works when game_phase is Bid"
        );
        if self.auction_properties_remaining() == 0 {
            return self.round_winner.unwrap();
        }
        let mut index = self.current_decision_player.unwrap();
        loop {
            if index + 1 == self.no_players {
                index = 0;
            } else {
                index += 1;
            }
            if self.active_players[index as usize] == true {
                return index;
            }
        }
    }
    pub fn next_game_phase(&mut self) {
        self.game_phase = GamePhase::Sell;
    }
    pub fn legal_moves(&self, player: Player) -> Vec<u8> {
        match self.game_phase() {
            GamePhase::Bid => self.legal_moves_bid(player).into_iter().collect(),
            GamePhase::Sell => self.legal_moves_sell(player).into_iter().collect(),
        }
    }
    pub fn legal_moves_bid(&self, player: Player) -> Vec<Coins> {
        debug_assert!(
            player < self.no_players,
            "Please ensure player is < {}. It is currently {}",
            self.no_players,
            player
        );
        let highest_bid = self.active_bids.iter().max().unwrap();
        let wealth: Coins = self.coins[player as usize];
        let player_current_bid: Coins = self.active_bids[player as usize];
        let mut actions: Vec<Coins> = Vec::with_capacity(wealth as usize);
        if wealth + player_current_bid < highest_bid + 1 {
            actions.push(0);
            return actions;
        }
        actions.push(0);
        for i in (highest_bid + 1 - player_current_bid)..=wealth {
            actions.push(i);
        }
        actions
    }
    pub fn legal_moves_sell(&self, player: Player) -> Vec<Property> {
        debug_assert!(
            player < self.no_players,
            "Please ensure player is < {}. It is currently {}",
            self.no_players,
            player
        );
        self.properties[&player].clone()
    }
    pub fn increase_bid(&mut self, player: Player, amount: Coins) {
        debug_assert!(
            player < self.no_players,
            "Please ensure player is < {}. It is currently {}",
            self.no_players,
            player
        );
        debug_assert!(
            self.active_players[player as usize] == true,
            "Trying to increase bid of a player who can no longer bid"
        );
        self.active_bids[player as usize] += amount;
    }
    pub fn take_card(&mut self, player: Player) {
        debug_assert!(
            self.auction_properties_remaining() > 0,
            "Cannot take_card if there are no auction properties to take"
        );
        let property: Property = self.auction_pool.pop().unwrap();
        self.insert_property_ascending(player, property);
        // if let Some(player_properties) = self.properties.get_mut(&player) {
        //     player_properties.push(self.auction_pool.pop().unwrap());
        // } else {
        //     debug_assert!(
        //         false,
        //         "Failed to find {player} in self.properties: {:?}",
        //         self.properties
        //     );
        // }
    }
    pub fn win_bid(&mut self, player: Player) {
        debug_assert!(
            self.auction_properties_remaining() > 0,
            "Cannot win_bid if there are no auction properties to give"
        );
        self.round_winner = Some(player);
        self.take_card(player);
        self.reset_round();
        if self.bid_phase_end() == false {
            self.reveal_auction(GamePhase::Bid);
        }
    }
    pub fn fold_bid(&mut self, player: Player) {
        debug_assert!(
            player < self.no_players,
            "Please ensure player is < {}. It is currently {}",
            self.no_players,
            player
        );
        debug_assert!(
            self.auction_properties_remaining() > 0,
            "Cannot fold_bid if there are no auction properties to give"
        );
        let coins_returned: Coins = self.active_bids[player as usize] / 2;
        self.add_coins(player, coins_returned);
        self.take_card(player);
        self.player_now_inactive(player);
    }
    pub fn raise_bid(&mut self, player: Player, amount: Coins) {
        debug_assert!(
            player < self.no_players,
            "Please ensure player is < {}. It is currently {}",
            self.no_players,
            player
        );
        debug_assert!(
            amount > 0,
            "Raise bid only works for amounts > 0. {amount} is not acceptable"
        );
        self.remove_coins(player, amount);
        self.increase_bid(player, amount);
    }
    pub fn reveal_auction(&mut self, game_phase: GamePhase) {
        if game_phase == GamePhase::Bid {
            debug_assert!(
                self.remaining_properties.len() > self.no_players as usize - 1,
                "Cannot reveal auction when you have {} properties remaining in the deck and {} players in total",
                self.remaining_properties.len(),
                self.no_players
            );
        } else {
            debug_assert!(
                self.remaining_checks.len() > self.no_players as usize - 1,
                "Cannot reveal auction when you have {} checks remaining in the deck and {} players in total",
                self.remaining_checks.len(),
                self.no_players
            );
        }
        debug_assert!(self.auction_pool.len() == 0, "Cannot reveal new auction while another auction has yet to end. Current auctio is: {:?}", self.auction_pool);
        match game_phase {
            GamePhase::Bid => {
                // for _ in 0..self.no_players {
                //     let new_property: Property = self.remaining_properties.pop().unwrap();
                //     self.auction_pool.push(new_property);
                // }
                self.auction_pool.extend(
                    self.remaining_properties
                        .drain(self.remaining_properties.len() - self.no_players as usize..),
                );
                self.auction_pool.sort_unstable_by(|a, b| b.cmp(a));
            }
            GamePhase::Sell => {
                if self.game_phase == GamePhase::Bid {
                    self.game_phase = GamePhase::Sell;
                }
                self.auction_pool.extend(
                    self.remaining_checks
                        .drain(self.remaining_checks.len() - self.no_players as usize..),
                );
                self.auction_pool.sort_unstable_by(|a, b| b.cmp(a));
            }
        }
    }
    pub fn generate_next_state_bid(&self, player: Player, action: Coins) -> Self {
        let mut new_state: GameState = self.clone();
        if action == 0 {
            // return coins and allocate property
            new_state.fold_bid(player);
            if new_state.auction_properties_remaining() == 1 {
                new_state.win_bid(self.next_player_bid());
            } else {
                new_state.current_decision_player = Some(new_state.next_player_bid());
            }
        } else {
            new_state.raise_bid(player, action);
            new_state.current_decision_player = Some(new_state.next_player_bid());
        }
        new_state
    }
    pub fn generate_next_state_sell(&mut self, player_choices: Vec<Property>) -> Self {
        debug_assert!(
            player_choices.len() == self.no_players as usize,
            "Length of player_choices should be {} not {}",
            self.no_players,
            player_choices.len()
        );
        let mut new_state = self.clone();
        new_state.active_bids = player_choices.clone();
        let mut player_bids: Vec<(Player, Property)> = (0..self.no_players)
            .map(|i| (i, player_choices[i as usize]))
            .collect();
        player_bids.sort_unstable_by(|a, b| a.1.cmp(&b.1));

        for (player, property) in player_bids.iter() {
            if let Some(check) = new_state.auction_pool.pop() {
                new_state.insert_check_ascending(*player, check);
            } else {
                debug_assert!(
                    false,
                    "Failed to pop from auction_pool: {:?}",
                    new_state.auction_pool
                );
            }
            if let Some(properties) = new_state.properties.get_mut(player) {
                properties.retain(|&x| x != *property);
            }
        }
        if new_state.remaining_checks.len() > 0 {
            new_state.reveal_auction(GamePhase::Sell);
        }
        new_state
    }
    pub fn bid_phase_end(&self) -> bool {
        if self.remaining_properties.len() == 0
            && (self.remaining_checks.len() < 30 || self.auction_pool.len() == 0)
        {
            // if self.remaining_properties.len() == 0 && self.auction_pool.len() == 0 {
            true
        } else {
            false
        }
    }
    pub fn sell_phase_end(&self) -> bool {
        if self.auction_pool.len() == 0
            && self.remaining_properties.len() == 0
            && self.remaining_checks.len() == 0
        {
            // if self.remaining_properties.len() == 0 && self.remaining_checks.len() == 0 {
            true
        } else {
            false
        }
    }
    pub fn tally_game_score(&self) -> String {
        let mut scores: AHashMap<Player, u8> = AHashMap::with_capacity(self.no_players as usize);

        // Sum checks for each player
        for (player, player_checks) in &self.checks {
            let score = player_checks.iter().copied().sum::<u8>();
            scores.insert(*player, score);
        }

        // Add coins to the player's score
        for (player, coins) in self.coins.iter().enumerate() {
            if let Some(score) = scores.get_mut(&(player as u8)) {
                *score += *coins;
            } else {
                scores.insert(player as u8, *coins);
            }
        }

        // Build the result string
        let mut result = String::new();
        for player in 0..self.no_players {
            let score = scores.get(&(player as u8)).unwrap_or(&0);
            write!(result, "Player {}: {} points\n", player + 1, score).unwrap();
        }

        result
    }
}

// impl fmt::Display for GameState {
//     fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
//         writeln!(f, "\nGameState Overview:")?;
//         writeln!(f, "------------------------------------")?;
//         writeln!(f, "--------- {} Auction ---------", self.game_phase)?;
//         writeln!(f, "------------------------------------")?;
//         writeln!(f, "      {:?}      ", self.auction_pool)?;
//         writeln!(f, "------------------------------------")?;
//         writeln!(f, "Player | Coins | Active Bids | Properties | Checks")?;
//         writeln!(f, "------------------------------------")?;
//         let empty_vec: Vec<Property> = vec![];
//         for player_index in 0..self.no_players {
//             let coins = self.coins.get(player_index as usize).unwrap_or(&0);
//             let active_bid = self.active_bids.get(player_index as usize).unwrap_or(&0);
//             let properties = self.properties.get(&player_index).unwrap_or(&empty_vec);
//             let checks = self.checks.get(&player_index).unwrap_or(&empty_vec);
//             writeln!(
//                 f,
//                 "Player {} | {} | {} | {:?} | {:?}",
//                 player_index + 1,
//                 coins,
//                 active_bid,
//                 properties,
//                 checks,
//             )?;
//         }
//         Ok(())
//     }
// }
impl fmt::Display for GameState {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f, "\nGameState Overview:")?;
        writeln!(f, "------------------------------------")?;
        writeln!(f, "--------- {} Auction ---------", self.game_phase)?;
        writeln!(f, "------------------------------------")?;
        writeln!(f, "      {:?}      ", self.auction_pool)?;
        writeln!(f, "------------------------------------")?;
        writeln!(
            f,
            "  Player | Bids/Sales | Coins |      Properties      | Checks"
        )?;
        writeln!(f, "------------------------------------")?;
        let empty_vec: Vec<Property> = vec![];
        for player_index in 0..self.no_players {
            let coins = self.coins.get(player_index as usize).unwrap_or(&0);
            let active_bid = self.active_bids.get(player_index as usize).unwrap_or(&0);
            let properties = self
                .properties
                .get(&(player_index as u8))
                .unwrap_or(&empty_vec);
            let checks = self.checks.get(&(player_index as u8)).unwrap_or(&empty_vec);
            writeln!(
                f,
                "Player {:<1} | {:<11} | {:<5} | {:<20} | {:?}",
                player_index + 1,
                active_bid,
                coins,
                format!("{:?}", properties), // Align the property vector
                checks,
            )?;
        }
        Ok(())
    }
}
