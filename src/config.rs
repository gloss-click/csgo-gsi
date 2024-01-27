use std::path::PathBuf;
use std::collections::{HashMap, HashSet};
use std::time::Duration;

use fehler::throws;

use crate::Error;

/// which pieces of information to subscribe to
///
/// [source](https://developer.valvesoftware.com/wiki/Counter-Strike:_Global_Offensive_Game_State_Integration#List_of_Gamestate_Integrations)
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Subscription {
    /// history of round wins
    MapRoundWins,
    /// mode, map, phase, team scores
    Map,
    /// steamid
    PlayerID,
    /// scoreboard info
    PlayerMatchStats,
    /// armor, flashed, equip_value, health, etc.
    PlayerState,
    /// list of player weapons and weapon state
    PlayerWeapons,
    /// info about the game providing info
    Provider,
    /// round phase and the winning team
    Round,
    /// grenade effecttime, lifetime, owner, position, type, velocity
    AllGrenades,
    /// the steam id of each player
    AllPlayersID,
    /// the scoreboard info for each player
    AllPlayersMatchStats,
    /// player_position but for each player
    AllPlayersPosition,
    /// the player_state for each player
    AllPlayersState,
    /// the player_weapons for each player
    AllPlayersWeapons,
    /// location of the bomb, who's carrying it, dropped or not
    Bomb,
    /// time remaining in tenths of a second, which phase
    PhaseCountdowns,
    /// forward direction, position for currently spectated player
    PlayerPosition,
}

impl Subscription {
    /// The subscriptions available in every context
    pub const UNRESTRICTED: &'static [Subscription] = &[
        Subscription::MapRoundWins,
        Subscription::Map,
        Subscription::PlayerID,
        Subscription::PlayerMatchStats,
        Subscription::PlayerState,
        Subscription::PlayerWeapons,
        Subscription::Provider,
        Subscription::Round,
    ];

    /// The subscriptions only available to spectators (**UNTESTED**)
    pub const SPECTATOR_ONLY: &'static [Subscription] = &[
        Subscription::AllGrenades,
        Subscription::AllPlayersID,
        Subscription::AllPlayersMatchStats,
        Subscription::AllPlayersPosition,
        Subscription::AllPlayersState,
        Subscription::AllPlayersWeapons,
        Subscription::Bomb,
        Subscription::PhaseCountdowns,
        Subscription::PlayerPosition,
    ];
}

impl From<&Subscription> for Subscription {
    fn from(x: &Subscription) -> Self {
        *x
    }
}

/// Builder struct for GSIConfig
#[derive(Clone)]
pub struct GSIConfigBuilder {
    name: String,
    timeout: Option<Duration>,
    buffer: Option<Duration>,
    throttle: Option<Duration>,
    heartbeat: Option<Duration>,
    auth: HashMap<String, String>,
    precision_time: Option<u8>,
    precision_position: Option<u8>,
    precision_vector: Option<u8>,
    subscriptions: HashSet<Subscription>,
}

impl GSIConfigBuilder {
    /// Initialize the builder, with the given service name
    pub fn new<S: Into<String>>(name: S) -> GSIConfigBuilder {
        GSIConfigBuilder {
            name: name.into(),
            timeout: None,
            buffer: None,
            throttle: None,
            heartbeat: None,
            auth: HashMap::new(),
            precision_time: None,
            precision_position: None,
            precision_vector: None,
            subscriptions: HashSet::new()
        }
    }

    /// CS:GO's client timeout for requests (default is 1.1 seconds)
    pub fn timeout(&mut self, timeout: Duration) -> &mut Self {
        self.timeout = Some(timeout);
        self
    }

    /// minimum wait between sending updates (default is 0.1 seconds)
    pub fn buffer(&mut self, buffer: Duration) -> &mut Self {
        self.buffer = Some(buffer);
        self
    }

    /// minimum wait between response to one update and sending the next (default is 1.0 seconds)
    pub fn throttle(&mut self, throttle: Duration) -> &mut Self {
        self.throttle = Some(throttle);
        self
    }

    /// maximum time between updates (default is 60 seconds)
    pub fn heartbeat(&mut self, heartbeat: Duration) -> &mut Self {
        self.heartbeat = Some(heartbeat);
        self
    }

    /// adds an authorization key/value pair (**not currently verified**)
    pub fn auth<S1: Into<String>, S2: Into<String>>(&mut self, key: S1, value: S2) -> &mut Self {
        self.auth.insert(key.into(), value.into());
        self
    }

    /// digits after the decimal point in time values (default is 2)
    pub fn precision_time(&mut self, precision: u8) -> &mut Self {
        self.precision_time = Some(precision);
        self
    }

    /// digits after the decimal point in position values (default is 2)
    pub fn precision_position(&mut self, precision: u8) -> &mut Self {
        self.precision_position = Some(precision);
        self
    }

    /// digits after the decimal point in vector values (default is 2)
    pub fn precision_vector(&mut self, precision: u8) -> &mut Self {
        self.precision_vector = Some(precision);
        self
    }

    /// subscribe to a certain set of update info
    pub fn subscribe(&mut self, subscription: Subscription) -> &mut Self {
        self.subscriptions.insert(subscription);
        self
    }

    /// subscribe to several sets of update info
    pub fn subscribe_multiple<I: IntoIterator<Item=S>, S: Into<Subscription>>(&mut self, subscriptions: I) -> &mut Self {
        self.subscriptions.extend(subscriptions.into_iter().map(|x| x.into()));
        self
    }

    /// create the config object
    pub fn build(&self) -> GSIConfig {
        GSIConfig::from(self)
    }
}

/// Game State Integration configuration
pub struct GSIConfig {
    service_name: String,
    timeout: Duration,
    buffer: Duration,
    throttle: Duration,
    heartbeat: Duration,
    auth: HashMap<String, String>,
    precision_time: u8,
    precision_position: u8,
    precision_vector: u8,
    subscriptions: HashSet<Subscription>,
}

impl From<GSIConfigBuilder> for GSIConfig {
    fn from(builder: GSIConfigBuilder) -> Self {
        GSIConfig {
            service_name: builder.name,
            timeout: builder.timeout.unwrap_or_else(|| Duration::from_secs_f64(1.1)),
            buffer: builder.buffer.unwrap_or_else(|| Duration::from_secs_f64(0.1)),
            throttle: builder.throttle.unwrap_or_else(|| Duration::from_secs_f64(1.0)),
            heartbeat: builder.heartbeat.unwrap_or_else(|| Duration::from_secs(60)),
            auth: builder.auth,
            precision_time: builder.precision_time.unwrap_or(2),
            precision_position: builder.precision_position.unwrap_or(2),
            precision_vector: builder.precision_vector.unwrap_or(2),
            subscriptions: builder.subscriptions,
        }
    }
}

impl From<&GSIConfigBuilder> for GSIConfig {
    fn from(builder: &GSIConfigBuilder) -> Self {
        Self::from(builder.clone())
    }
}

impl GSIConfig {
    #[throws]
    pub(crate) fn install_into<P: Into<PathBuf>>(&self, cfg_folder: P, port: u16) {
        let mut cfg_path = cfg_folder.into();
        cfg_path.push(&format!("gamestate_integration_{}.cfg", &self.service_name));
        let config = config_file::ConfigFile::new(&self, port);
        let config = vdf_serde::to_string(&config)
            .map_err(|err| Error::ConfigInstallError { description: "failed to serialize config for installation", cause: Some(Box::new(err)) })?;
        ::std::fs::write(cfg_path, config.as_bytes())
            .map_err(|err| Error::ConfigInstallError { description: "failed to write config file", cause: Some(Box::new(err)) })?;
    }
}

mod config_file {
    use std::collections::HashMap;

    use serde::Serialize;
    use crate::config::GSIConfig;

    #[derive(Serialize)]
    struct Precision {
        precision_time: u8,
        precision_position: u8,
        precision_vector: u8,
    }

    #[derive(Serialize)]
    struct Data {
        map_round_wins: bool,
        map: bool,
        player_id: bool,
        player_match_stats: bool,
        player_state: bool,
        player_weapons: bool,
        provider: bool,
        round: bool,

        // Below this line must be spectating or observing
        allgrenades: bool,
        allplayers_id: bool,
        allplayers_match_stats: bool,
        allplayers_position: bool,
        allplayers_state: bool,
        allplayers_weapons: bool,
        bomb: bool,
        phase_countdowns: bool,
        player_position: bool,
    }

    #[derive(Serialize)]
    #[serde(rename = "Managed by the csgo-gsi Rust library")]
    pub struct ConfigFile {
        uri: String,
        timeout: f64,
        buffer: f64,
        throttle: f64,
        heartbeat: f64,
        auth: HashMap<String, String>,
        output: Precision,
        data: Data,
    }

    impl ConfigFile {
        pub fn new(config: &GSIConfig, port: u16) -> Self {
            use super::Subscription;
            ConfigFile {
                uri: format!("http://127.0.0.1:{}", port),
                timeout: config.timeout.as_secs_f64(),
                buffer: config.buffer.as_secs_f64(),
                throttle: config.throttle.as_secs_f64(),
                heartbeat: config.heartbeat.as_secs_f64(),
                auth: config.auth.clone(),
                output: Precision {
                    precision_time: config.precision_time,
                    precision_position: config.precision_position,
                    precision_vector: config.precision_vector,
                },
                data: Data {
                    map_round_wins: config.subscriptions.contains(&Subscription::MapRoundWins),
                    map: config.subscriptions.contains(&Subscription::Map),
                    player_id: config.subscriptions.contains(&Subscription::PlayerID),
                    player_match_stats: config.subscriptions.contains(&Subscription::PlayerMatchStats),
                    player_state: config.subscriptions.contains(&Subscription::PlayerState),
                    player_weapons: config.subscriptions.contains(&Subscription::PlayerWeapons),
                    provider: config.subscriptions.contains(&Subscription::Provider),
                    round: config.subscriptions.contains(&Subscription::Round),
                    allgrenades: config.subscriptions.contains(&Subscription::AllGrenades),
                    allplayers_id: config.subscriptions.contains(&Subscription::AllPlayersID),
                    allplayers_match_stats: config.subscriptions.contains(&Subscription::AllPlayersMatchStats),
                    allplayers_position: config.subscriptions.contains(&Subscription::AllPlayersPosition),
                    allplayers_state: config.subscriptions.contains(&Subscription::AllPlayersState),
                    allplayers_weapons: config.subscriptions.contains(&Subscription::AllPlayersWeapons),
                    bomb: config.subscriptions.contains(&Subscription::Bomb),
                    phase_countdowns: config.subscriptions.contains(&Subscription::PhaseCountdowns),
                    player_position: config.subscriptions.contains(&Subscription::PlayerPosition),
                },
            }
        }
    }
}
