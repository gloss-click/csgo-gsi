use std::path::PathBuf;
use std::sync::mpsc;
use std::task::Poll;
use std::time::Duration;

use fehler::throws;
use futures::future::BoxFuture;
use futures::FutureExt;
use log::info;

use warp::Filter;

use crate::{update, Error, GSIConfig, Update};

/// the event that signals to teardown the gsi server
#[derive(Copy, Clone, Debug)]
pub struct CloseEvent{}

/// a server that listens for GSI updates
pub struct GSIServer {
    port: u16,
    config: GSIConfig,
    listeners: Vec<Box<dyn FnMut(&update::Update) + Send>>,
}

impl GSIServer {
    /// create a new server with the given configuration and port
    pub fn new(config: GSIConfig, port: u16) -> Self {
        Self {
            port,
            config,
            listeners: vec![],
        }
    }

    /// remove all attached listeners
    pub fn close(&mut self) {
        self.listeners.clear();
    }

    /// install this server's configuration into the given `/path/to/csgo/cfg/` folder
    #[throws]
    pub fn install_into<P: Into<PathBuf> + std::fmt::Debug>(&mut self, cfg_folder: P) {
        self.config.install_into(cfg_folder, self.port)?;
    }

    /// install this server's configuration into the autodiscovered `/path/to/csgo/cfg/` folder, if it can be found
    #[throws]
    pub fn install(&mut self, path: PathBuf) {
        self.install_into(path)?;
    }

    /// add an update listener
    pub fn add_listener<F: 'static + FnMut(&update::Update) + Send>(&mut self, listener: F) {
        self.listeners.push(Box::new(listener));
    }

    /// run the server (will block indefinitely)
    #[throws]
    pub async fn run(mut self, tokio_handle: tokio::runtime::Handle, mut close_event: tokio::sync::broadcast::Receiver<CloseEvent>) {
        let (tx, rx) = mpsc::sync_channel(128);

        let port = self.port;

        info!("Launching GSI server");
        
        let gsi = warp::post()
        .and(warp::body::content_length_limit(1024*16))
        .and(warp::body::json())
        .map(move |update: Update| {
            let _ = tx.send(update);
            warp::reply()
        });
        
        let (_sockaddr, thandle) = warp::serve(gsi).bind_with_graceful_shutdown(([127, 0, 0, 1], port), async move { close_event.recv().await; });
        let handle = tokio_handle.spawn(thandle);
        
        loop {
            let update = rx.recv();
            match update {
                Ok(u) => {
                    for callback in &mut self.listeners {
                        callback(&u)
                    }
                },
                Err(_) => {
                    break;
                },
            }
        }
            
        self.close();

        handle.abort();
        handle.await.expect("Error joining GSI server inner thread.");

        info!("Ending GSI server thread");
    }
}
