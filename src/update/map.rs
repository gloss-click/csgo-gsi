//! map-related information

use std::collections::HashMap;

use serde::{Serialize, Deserialize};

/// map information
#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct Map {
    /// game mode
    pub mode: Mode,
    /// map name
    pub name: String,
    /// number of matches to win series
    pub num_matches_to_win_series: u64,
    /// map phase
    pub phase: Phase,
    /// current round number
    pub round: u64,
    /// who won which round and how
    #[serde(default)]
    pub round_wins: HashMap<u64, RoundWin>,
    /// counter-terrorist team info
    pub team_ct: Team,
    /// terrorist team info
    pub team_t: Team,
}

/// game mode
#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum Mode {
    /// competitive
    Competitive,
    /// casual
    Casual,
    /// deathmatch
    Deathmatch,
    /// the tutorial
    Training,
    /// demolition
    #[serde(rename = "gungametrbomb")]
    Demolition,
    /// arms race
    #[serde(rename = "gungameprogressive")]
    ArmsRace,
    /// wingman
    #[serde(rename = "scrimcomp2v2")]
    Wingman,
}

/// map phase
#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum Phase {
    /// warmup
    Warmup,
    /// in game
    Live,
    /// halftime
    Intermission,
    /// game over
    GameOver,
}

/// information about who won and how
/// ("ct_win_time", "t_win_bomb", "ct_win_elimination", "ct_win_defuse", etc)
/// (TODO actually parse)
pub type RoundWin = String;

/// team info
#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct Team {
    /// rounds won
    pub score: u64,
    /// rounds lost in a row
    pub consecutive_round_losses: u64,
    /// timeouts remaining
    pub timeouts_remaining: u64,
    /// matches won this series
    pub matches_won_this_series: u64,
    /// team name
    pub name: Option<String>,
    /// flag code (TODO find options)
    pub flag: Option<String>,
}
