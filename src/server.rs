use std::path::PathBuf;
use std::sync::mpsc;

use fehler::throws;
use futures::future::BoxFuture;
use gotham::handler::HandlerError;
use gotham::helpers::http::response::create_empty_response;
use gotham::hyper::{body, Body, Response, StatusCode};
use gotham::middleware::state::StateMiddleware;
use gotham::pipeline::single_pipeline;
use gotham::pipeline::single_middleware;
use gotham::router::builder::{DefineSingleRoute, build_router};
use gotham::router::builder::DrawRoutes;
use gotham::router::Router;
use gotham::state::{State, FromState};
use log::info;

use crate::{GSIConfig, Error, update};

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
    pub async fn run(mut self, exit_signal: BoxFuture<'static, Result<(), Error>>) {
        let (tx, rx) = mpsc::sync_channel(128);

        let port = self.port;

        let gblock = async move {
            info!("Launching GSI server");
            let fut = gotham::init_server(("127.0.0.1", port), router(tx));

            tokio::select! {
                _ = fut => {
                    info!("GSI server exiting");
                },
                _ = exit_signal => {
                    info!("GSI server received exit signal");
                },
            };
        };

        let handle = tokio::spawn(gblock);

        for update in rx {
            for callback in &mut self.listeners {
                callback(&update)
            }
        }

        self.close();

        handle.await.expect("Error joining GSI server inner thread.");

        info!("Ending GSI server thread");
    }
}

#[derive(Clone, StateData)]
struct UpdateHandler {
    inner: mpsc::SyncSender<update::Update>,
}

impl UpdateHandler {
    fn new(tx: &mpsc::SyncSender<update::Update>) -> Self {
        Self {
            inner: tx.clone()
        }
    }

    fn send(&self, update: update::Update) {
        self.inner.send(update).expect("failed to send update back to main thread");
    }
}

#[throws((State, HandlerError))]
pub async fn handle_update(mut state: State) -> (State, Response<Body>) {
    let body = state.try_take::<Body>();
    let body = match body {
        Some(body) => body,
        None => {
            let response = create_empty_response(&state, StatusCode::BAD_REQUEST);
            return (state, response);
        }
    };
    let body = body::to_bytes(body).await;
    let body = match body {
        Ok(body) => body,
        Err(err) => {
            log::warn!("{}", err);
            let response = create_empty_response(&state, StatusCode::INTERNAL_SERVER_ERROR);
            return (state, response);
        }
    };
    let json_value = serde_json::from_slice::<serde_json::Value>(body.as_ref());
    let json_value = match json_value {
        Ok(json_value) => json_value,
        Err(err) => {
            log::warn!("JSON parsing error: {}", err);
            if let Ok(data) = ::std::str::from_utf8(body.as_ref()) {
                log::warn!("{}\n", data);
            }
            let response = create_empty_response(&state, StatusCode::INTERNAL_SERVER_ERROR);
            return (state, response);
        }
    };
    let data = serde_json::from_value::<update::Update>(json_value);
    let data = match data {
        Ok(data) => data,
        Err(err) => {
            log::warn!("Update parsing error: {}", err);
            if let Ok(data) = ::std::str::from_utf8(body.as_ref()) {
                log::warn!("{}\n", data);
            }
            let response = create_empty_response(&state, StatusCode::INTERNAL_SERVER_ERROR);
            return (state, response);
        }
    };
    // TODO verify auth
    {
        let update_handler = UpdateHandler::borrow_from(&state);
        update_handler.send(data);
    }
    let response = create_empty_response(&state, StatusCode::OK);
    (state, response)
}

fn router(tx: mpsc::SyncSender<update::Update>) -> Router {
    let update_handler = UpdateHandler::new(&tx);

    let middleware = StateMiddleware::new(update_handler);
    let pipeline = single_middleware(middleware);
    let (chain, pipelines) = single_pipeline(pipeline);

    build_router(chain, pipelines, |route| {
        route
            .post("/")
            .to_async(handle_update);
    })
}
