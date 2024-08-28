use crate::models::enums::{Check, Coins, GamePhase, Player, Property};
use ahash::AHasher;
use ahash::{AHashMap, RandomState};
use itertools::Itertools;
use rand::seq::{IndexedRandom, SliceRandom};
use rand::thread_rng;
use std::fmt;
use std::fmt::Write;
use std::hash::{BuildHasher, Hash, Hasher};

#[derive(Clone, Debug)]
pub struct GameState {
    game_phase: GamePhase,
    no_players: Player,
    coins: Vec<Coins>,
    properties: AHashMap<Player, Vec<Property>>,
    checks: AHashMap<Player, Vec<Check>>,
    previous_decision_player: Option<u8>, // TODO: Validate previous player update logic
    current_decision_player: Option<u8>,
    active_players: Vec<bool>,
    active_bids: Vec<u8>,
    auction_pool: Vec<u8>,
    remaining_properties: Vec<Property>,
    remaining_checks: Vec<Check>,
    round_winner: Option<Player>,
    round_no: u8,
    turn_no: u32,
    parent_hash: u64,
}

impl GameState {
    pub fn starting(no_players: u8, starting_player: u8) -> Self {
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
        let current_decision_player: Option<u8> = Some(starting_player);
        GameState {
            game_phase: GamePhase::Bid,
            no_players,
            coins,
            properties,
            checks,
            previous_decision_player: None,
            current_decision_player,
            active_players,
            active_bids,
            auction_pool: auction_properties,
            remaining_properties,
            remaining_checks,
            round_winner: None,
            round_no: 0, // 0 because it is incremented in reveal_auction
            turn_no: 1,
            parent_hash: 0,
        }
    }
    pub fn previous_player(&self) -> Player {
        self.previous_decision_player.unwrap()
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
    pub fn turn_no(&self) -> u32 {
        self.turn_no
    }
    pub fn add_turn_no(&mut self, amount: u32) {
        self.turn_no += amount;
    }
    pub fn round_no(&self) -> u8 {
        self.round_no
    }
    pub fn add_round_no(&mut self, amount: u8) {
        self.round_no += amount;
    }
    pub fn no_players(&self) -> u8 {
        self.no_players
    }
    pub fn get_player_properties(&self, player: Player) -> &Vec<Property> {
        debug_assert!(
            player < self.no_players,
            "Player number of {player} too high! Keep it less than {}",
            &self.no_players
        );
        &self.properties[&player]
    }
    pub fn get_player_coins(&self, player: Player) -> Coins {
        debug_assert!(
            player < self.no_players,
            "Player number of {player} too high! Keep it less than {}",
            &self.no_players
        );
        self.coins[player as usize]
    }
    pub fn get_player_checks(&self, player: Player) -> &Vec<Check> {
        debug_assert!(
            player < self.no_players,
            "Player number of {player} too high! Keep it less than {}",
            &self.no_players
        );
        &self.checks[&player]
    }
    pub fn get_remaining_checks(&self) -> &Vec<Check> {
        &self.remaining_checks
    }
    pub fn get_remaining_properties(&self) -> &Vec<Property> {
        &self.remaining_checks
    }
    pub fn get_coins(&self) -> &Vec<Coins> {
        &self.coins
    }
    pub fn auction_end(&self) -> bool {
        if self.auction_pool.len() == 0 {
            true
        } else {
            false
        }
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
        self.previous_decision_player = self.current_decision_player;
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
    pub fn get_hash(&self) -> u64 {
        // Create an instance of AHasher with specific keys
        let mut hasher = RandomState::with_seeds(12345, 67890, 11111, 22222).build_hasher();

        // Hash the GameState
        self.hash(&mut hasher);

        // Return the computed hash value
        hasher.finish()
    }
    pub fn get_parent_hash(&self) -> u64 {
        self.parent_hash
    }
    pub fn set_parent_hash(&mut self, parent_hash: u64) {
        self.parent_hash = parent_hash;
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
    }
    pub fn win_bid(&mut self, player: Player) {
        debug_assert!(
            self.auction_properties_remaining() > 0,
            "Cannot win_bid if there are no auction properties to give"
        );
        self.round_winner = Some(player);
        self.take_card(player);
        self.reset_round();
        // if self.bid_phase_end() == false {
        //     self.reveal_auction(GamePhase::Bid);
        // }
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
        self.active_bids[player as usize] = 0;
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
    pub fn reveal_auction(&mut self) {
        if self.game_phase == GamePhase::Bid {
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
        match self.game_phase {
            GamePhase::Bid => {
                self.auction_pool.extend(
                    self.remaining_properties
                        .drain(self.remaining_properties.len() - self.no_players as usize..),
                );
                self.auction_pool.sort_unstable_by(|a, b| b.cmp(a));
            }
            GamePhase::Sell => {
                // if self.game_phase == GamePhase::Bid {
                //     self.game_phase = GamePhase::Sell;
                // }
                self.auction_pool.extend(
                    self.remaining_checks
                        .drain(self.remaining_checks.len() - self.no_players as usize..),
                );
                self.auction_pool.sort_unstable_by(|a, b| b.cmp(a));
            }
        }
    }
    pub fn reveal_auction_perms(&self, random_sample: bool, n_sample: u32) -> Vec<Self> {
        // TODO: Validate this function
        debug_assert!(self.auction_pool.len() == 0, "Cannot reveal new auction while another auction has yet to end. Current auctio is: {:?}", self.auction_pool);
        let mut results: Vec<GameState> = Vec::new();
        let parent_hash: u64 = self.get_hash();
        match self.game_phase {
            GamePhase::Bid => {
                debug_assert!(
                    self.remaining_properties.len() > self.no_players as usize - 1,
                    "Cannot reveal auction when you have {} properties remaining in the deck and {} players in total",
                    self.remaining_properties.len(),
                    self.no_players
                );
                if random_sample {
                    let mut rng = thread_rng();
                    for _ in 0..n_sample {
                        let mut cloned_state = self.clone();
                        let sampled_properties: Vec<u8> = cloned_state
                            .remaining_properties
                            .choose_multiple(&mut rng, self.no_players as usize)
                            .cloned()
                            .collect();
                        cloned_state
                            .remaining_properties
                            .retain(|prop| !sampled_properties.contains(prop));
                        cloned_state.auction_pool.extend(sampled_properties);
                        cloned_state.auction_pool.sort_unstable_by(|a, b| b.cmp(a));
                        cloned_state.set_parent_hash(parent_hash);
                        results.push(cloned_state);
                    }
                } else {
                    let combinations = self
                        .remaining_properties
                        .iter()
                        .combinations(self.no_players as usize);

                    for combination in combinations {
                        let mut cloned_state = self.clone();
                        let sampled_properties: Vec<u8> =
                            combination.into_iter().cloned().collect();
                        cloned_state
                            .remaining_properties
                            .retain(|prop| !sampled_properties.contains(prop));
                        cloned_state.auction_pool.extend(sampled_properties);
                        cloned_state.auction_pool.sort_unstable_by(|a, b| b.cmp(a));
                        cloned_state.set_parent_hash(parent_hash);
                        results.push(cloned_state);
                    }
                }
            }
            GamePhase::Sell => {
                debug_assert!(
                    self.remaining_checks.len() > self.no_players as usize - 1,
                    "Cannot reveal auction when you have {} checks remaining in the deck and {} players in total",
                    self.remaining_checks.len(),
                    self.no_players
                );
                if random_sample {
                    let mut rng = thread_rng();
                    for _ in 0..n_sample {
                        let mut cloned_state = self.clone();
                        let sampled_properties: Vec<u8> = cloned_state
                            .remaining_checks
                            .choose_multiple(&mut rng, self.no_players as usize)
                            .cloned()
                            .collect();
                        cloned_state
                            .remaining_checks
                            .retain(|prop| !sampled_properties.contains(prop));
                        cloned_state.auction_pool.extend(sampled_properties);
                        cloned_state.auction_pool.sort_unstable_by(|a, b| b.cmp(a));
                        cloned_state.set_parent_hash(parent_hash);
                        results.push(cloned_state);
                    }
                } else {
                    let combinations = self
                        .remaining_checks
                        .iter()
                        .combinations(self.no_players as usize);

                    for combination in combinations {
                        let mut cloned_state = self.clone();
                        let sampled_properties: Vec<u8> =
                            combination.into_iter().cloned().collect();
                        cloned_state
                            .remaining_checks
                            .retain(|prop| !sampled_properties.contains(prop));
                        cloned_state.auction_pool.extend(sampled_properties);
                        cloned_state.auction_pool.sort_unstable_by(|a, b| b.cmp(a));
                        cloned_state.set_parent_hash(parent_hash);
                        results.push(cloned_state);
                    }
                }
            }
        }
        results
    }
    pub fn generate_next_state_bid(&self, player: Player, action: Coins) -> Self {
        if self.auction_end() {
            let mut new_state: GameState = self.clone();
            let parent_hash = self.get_hash();
            new_state.reveal_auction();
            new_state.set_parent_hash(parent_hash);
            new_state
        } else {
            self.manual_next_state_bid(player, action)
        }
    }
    pub fn generate_next_state_sell(&mut self, player_choices: Vec<Property>) -> Self {
        debug_assert!(
            player_choices.len() == self.no_players as usize,
            "Length of player_choices should be {} not {}",
            self.no_players,
            player_choices.len()
        );
        if self.auction_end() {
            let mut new_state = self.clone();
            let parent_hash = self.get_hash();
            new_state.reveal_auction();
            new_state.active_bids = vec![0; 6];
            new_state.set_parent_hash(parent_hash);
            new_state
        } else {
            self.manual_next_state_sell(player_choices)
        }
    }
    pub fn manual_next_state_bid(&self, player: Player, action: Coins) -> Self {
        let mut new_state: GameState = self.clone();
        let parent_hash = self.get_hash();
        new_state.set_parent_hash(parent_hash);
        if action == 0 {
            // return coins and allocate property
            new_state.fold_bid(player);
            if new_state.auction_properties_remaining() == 1 {
                new_state.win_bid(self.next_player_bid());
            } else {
                new_state.previous_decision_player = new_state.current_decision_player;
                new_state.current_decision_player = Some(new_state.next_player_bid());
            }
        } else {
            new_state.raise_bid(player, action);
            new_state.previous_decision_player = new_state.current_decision_player;
            new_state.current_decision_player = Some(new_state.next_player_bid());
        }
        new_state.add_turn_no(1);
        if new_state.auction_end() {
            new_state.add_round_no(1);
            if new_state.remaining_properties.len() == 0 {
                new_state.game_phase = GamePhase::Sell;
            }
        }
        new_state
    }
    pub fn manual_next_state_sell(&mut self, player_choices: Vec<Property>) -> Self {
        debug_assert!(
            player_choices.len() == self.no_players as usize,
            "Length of player_choices should be {} not {}",
            self.no_players,
            player_choices.len()
        );
        let mut new_state = self.clone();
        let parent_hash = self.get_hash();
        new_state.set_parent_hash(parent_hash);
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
        new_state.add_turn_no(1);
        new_state.add_round_no(1);
        new_state
    }
    pub fn bid_phase_end(&self) -> bool {
        if self.remaining_properties.len() == 0
            && (self.remaining_checks.len() < 30 || self.auction_pool.len() == 0)
        {
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

impl Hash for GameState {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.coins.hash(state); // Hash the coins vector
        for key in 0..6u8 {
            if let Some(vec) = self.properties.get(&key) {
                vec.hash(state);
            }
        }
        for key in 0..6u8 {
            if let Some(vec) = self.checks.get(&key) {
                vec.hash(state);
            }
        }
        self.auction_pool.hash(state); // Hash the auction_pool vector
    }
}

impl fmt::Display for GameState {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f, "\nGameState Overview:")?;
        writeln!(
            f,
            "----Round: {}--- Turn: {}--",
            self.round_no, self.turn_no
        )?;
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
