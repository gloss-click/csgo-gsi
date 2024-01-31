//! all information that can be contained in a GSI update

use std::collections::HashMap;

use serde::{Serialize, Deserialize, de::IgnoredAny};

pub mod player;
use player::Player;

pub mod map;
use map::Map;

pub mod round;
use round::Round;

// TODO abuse generics to align subscriptions with these types

/// a team
#[derive(Clone, Debug, Deserialize, Serialize)]
pub enum Team {
    /// counter-terrorists
    CT,
    /// terrorists
    T,
}

/// an update received from CS:GO
#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct Update {
    /// map info
    pub map: Option<Map>,
    /// player info
    pub player: Option<Player>,
    /// provider (CS:GO) info
    pub provider: Option<Provider>,
    /// authentication info, matching initial config
    pub auth: HashMap<String, String>,
    /// round info
    pub round: Option<Round>,
    #[serde(skip_serializing, default)]
    added: IgnoredAny,
    #[serde(skip_serializing, default)]
    previously: IgnoredAny,
}

/// information about the GSI info provider (CS:GO itself)
#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct Provider {
    /// game name
    pub name: String,
    /// steam app ID
    #[serde(rename = "appid")]
    pub app_id: u64,
    /// version number
    pub version: u64,
    /// player's steam ID
    #[serde(rename = "steamid")]
    pub steam_id: String,
    /// update timestamp
    pub timestamp: u64,
}

#[cfg(feature = "rhai")]
#[allow(missing_docs)]
mod rhai_package {
    use std::fmt::Display;
    use rhai::{
        def_package,
        Dynamic,
        ImmutableString,
        Map as RhaiMap,
        packages::CorePackage,
    };
    use super::*;

    trait OptionExt {
        fn into_dynamic(self) -> Dynamic;
    }

    impl<T: Send + Sync + Clone + 'static> OptionExt for Option<T> {
        fn into_dynamic(self) -> Dynamic {
            match self {
                Some(x) => Dynamic::from(x),
                None => Dynamic::from(()),
            }
        }
    }

    trait StringExt {
        fn into_immutable(self) -> ImmutableString;
    }

    impl StringExt for String {
        fn into_immutable(self) -> ImmutableString {
            self.into()
        }
    }

    trait MapExt {
        fn into_rhai_map(self) -> RhaiMap;
    }

    impl<K: Display, V: 'static + Send + Sync + Clone> MapExt for HashMap<K, V> {
        fn into_rhai_map(self) -> RhaiMap {
            self.into_iter()
                .map(|(k, v)| -> (ImmutableString, Dynamic) {
                    (format!("{}", k).into(), Dynamic::from(v))
                })
                .collect()
        }
    }

    def_package!(rhai:CSGOPackage:"Rhai package for CS:GO GSI updates", module, {
        CorePackage::init(module);

        macro_rules! make_debug {
            ($ty:ty) => {
                module.set_fn_1_mut("debug", |x: &mut $ty| -> Result<ImmutableString, _> {
                    Ok(format!("{:?}", x).into())
                });
            }
        }

        macro_rules! make_stringable {
            ($ty:ty as Debug) => {
                make_debug!($ty);
                module.set_fn_1_mut("to_string", |x: &mut $ty| -> Result<ImmutableString, _> {
                    Ok(format!("{:?}", x).into())
                });
                module.set_fn_1_mut("print", |x: &mut $ty| -> Result<ImmutableString, _> {
                    Ok(format!("{:?}", x).into())
                });
                module.set_fn_2("+", |s1: ImmutableString, s: $ty| -> Result<ImmutableString, _> {
                    Ok(s1 + format!("{:?}", s))
                });
                module.set_fn_2("+", |s: $ty, s2: ImmutableString| -> Result<ImmutableString, _> {
                    Ok(format!("{:?}{}", s, s2).into())
                });
                module.set_fn_2_mut("+=", |s1: &mut ImmutableString, s: $ty| -> Result<(), _> {
                    *s1 = format!("{}{:?}", s1, s).into();
                    Ok(())
                })
            }
        }

        macro_rules! make_getter {
            ($ty:ty:type.$into:ident()) => {
                module.set_getter_fn("type", |x: &mut $ty| Ok(x.r#type.clone().$into()));
            };

            ($ty:ty:$name:ident) => {
                module.set_getter_fn(stringify!($name), |x: &mut $ty| Ok(x.$name.clone()));
            };

            ($ty:ty:$name:ident.$into:ident()) => {
                module.set_getter_fn(stringify!($name), |x: &mut $ty| Ok(x.$name.clone().$into()));
            };
        }

        make_debug!(Update);
        make_getter!(Update:map.into_dynamic());
        make_getter!(Update:player.into_dynamic());
        make_getter!(Update:provider.into_dynamic());
        make_getter!(Update:round.into_dynamic());

        make_debug!(Provider);
        make_getter!(Provider:name.into_immutable());
        make_getter!(Provider:app_id);
        make_getter!(Provider:version);
        make_getter!(Provider:steam_id.into_immutable());
        make_getter!(Provider:timestamp);

        make_stringable!(Team as Debug);

        make_debug!(Map);
        make_getter!(Map:mode);
        make_getter!(Map:name.into_immutable());
        make_getter!(Map:num_matches_to_win_series);
        make_getter!(Map:phase);
        make_getter!(Map:round);
        make_getter!(Map:round_wins.into_rhai_map());
        make_getter!(Map:team_ct);
        make_getter!(Map:team_t);

        make_stringable!(map::Mode as Debug);

        make_stringable!(map::Phase as Debug);

        make_debug!(map::Team);
        make_getter!(map::Team:score);
        make_getter!(map::Team:consecutive_round_losses);
        make_getter!(map::Team:timeouts_remaining);
        make_getter!(map::Team:matches_won_this_series);
        make_getter!(map::Team:name.into_dynamic());
        make_getter!(map::Team:flag.into_dynamic());

        make_debug!(Player);
        make_getter!(Player:steam_id);
        make_getter!(Player:name);
        make_getter!(Player:observer_slot.into_dynamic());
        make_getter!(Player:activity);
        make_getter!(Player:match_stats.into_dynamic());
        make_getter!(Player:state.into_dynamic());
        make_getter!(Player:team.into_dynamic());
        make_getter!(Player:weapons.into_rhai_map());
        make_getter!(Player:clan.into_dynamic());

        make_stringable!(player::Activity as Debug);

        make_debug!(player::MatchStats);
        make_getter!(player::MatchStats:kills);
        make_getter!(player::MatchStats:assists);
        make_getter!(player::MatchStats:deaths);
        make_getter!(player::MatchStats:mvps);
        make_getter!(player::MatchStats:score);

        make_debug!(player::State);
        make_getter!(player::State:health);
        make_getter!(player::State:armor);
        make_getter!(player::State:helmet);
        make_getter!(player::State:flashed);
        make_getter!(player::State:smoked);
        make_getter!(player::State:burning);
        make_getter!(player::State:money);
        make_getter!(player::State:round_kills);
        make_getter!(player::State:round_killhs);
        make_getter!(player::State:equip_value);
        make_getter!(player::State:round_totaldmg.into_dynamic());
        make_getter!(player::State:defuse_kit.into_dynamic());

        make_debug!(player::Weapon);
        make_getter!(player::Weapon:name);
        make_getter!(player::Weapon:paintkit);
        make_getter!(player::Weapon:type.into_dynamic());
        make_getter!(player::Weapon:state);
        make_getter!(player::Weapon:ammo_clip.into_dynamic());
        make_getter!(player::Weapon:ammo_clip_max.into_dynamic());
        make_getter!(player::Weapon:ammo_reserve.into_dynamic());

        make_stringable!(player::WeaponType as Debug);

        make_stringable!(player::WeaponState as Debug);

        make_debug!(Round);
        make_getter!(Round:phase);
        make_getter!(Round:bomb.into_dynamic());
        make_getter!(Round:win_team.into_dynamic());

        make_stringable!(round::Phase as Debug);

        make_stringable!(round::BombState as Debug);
    });
}

#[cfg(feature = "rhai")]
pub use rhai_package::CSGOPackage;
