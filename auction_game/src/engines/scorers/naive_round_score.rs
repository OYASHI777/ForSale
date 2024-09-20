use crate::engines::controllers::constants::VALUE_PER_PROPERTY;
use crate::models::enums::{GamePhase, Player, Property};
use crate::models::game_state::GameState;
use ahash::AHashMap;
use num_traits::float::FloatCore;

pub struct NaiveRoundScore {}
impl NaiveRoundScore {
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
