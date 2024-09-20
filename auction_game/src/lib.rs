pub mod engines {
    pub mod controllers {
        pub mod constants;
        pub mod counterfactual_regret;
        pub mod greedy_baby;
        pub mod maxn_player;
        pub mod maxn_player_multi;
        pub mod random_player;
        pub mod terminal_player;
    }
    pub mod q_values {
        pub mod regret;
    }
    pub mod scorers {
        pub mod naive_round_score;
    }
    pub mod strategies {
        pub mod average;
    }
    pub mod traits;
    pub mod utils;
}
pub mod game_modes {
    pub mod play_with_bots;
    pub mod self_play;
    pub mod standard;
    pub mod traits;
}
pub mod helper {}
pub mod models {
    pub mod constants;
    pub mod enums;
    pub mod game_state;
}
