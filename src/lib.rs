//! Helper library for the [CS:GO Game State Integration (GSI) API][gsi].
//!
//! Best used with the [tokio](https://tokio.rs/) async ecosystem.
//!
//! [gsi]: https://developer.valvesoftware.com/wiki/Counter-Strike:_Global_Offensive_Game_State_Integration
//!
//! # Simple Example
//!
//! ```no_run
//! use csgo_gsi::{GSIConfigBuilder, GSIServer, Subscription};
//!
//! #[tokio::main]
//! async fn main() {
//!     let config = GSIConfigBuilder::new("csgo-gsi Example")
//!         .subscribe_multiple(Subscription::UNRESTRICTED)
//!         .build();
//!
//!     let mut server = GSIServer::new(config, 31337);
//!     server.add_listener(|update| println!("Got an update {:#?}", update));
//!
//!     server
//!         .run()
//!         .await
//!         .expect("server didn't start");
//! }
//! ```
#![deny(missing_docs)]
#![doc(html_root_url = "https://docs.rs/csgo-gsi/0.3.0")]

#[macro_use]
extern crate gotham_derive;

mod config;
mod error;
mod server;
pub mod update;

pub use config::{Subscription, GSIConfigBuilder, GSIConfig};
pub use error::Error;
pub use server::GSIServer;
pub use update::Update;
