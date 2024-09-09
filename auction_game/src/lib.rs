pub mod engines {
    pub mod controllers {
        pub mod constants;
        pub mod maxn_player;
        pub mod maxn_player_multi;
        pub mod random_player;
    }
    pub mod traits;
}
pub mod game_modes {
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
