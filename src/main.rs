use std::collections::HashMap;
use std::net::SocketAddr;
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

use axum::extract::{Path, Query, State};
use axum::http::{HeaderMap, StatusCode};
use axum::Router;
use clap::Parser;
use tokio::net::TcpListener;

#[derive(Debug, Clone, PartialEq, Eq)]
struct ServiceState {
    name: String,
    deadline: Instant,
}

struct AppState {
    write_token: String,
    services: Mutex<Vec<ServiceState>>,
}

const TOKEN_HEADER: &str = "x-write-token";

async fn ping(
    State(state): State<Arc<AppState>>,
    Path(name): Path<String>,
    Query(params): Query<HashMap<String, String>>,
    headers: HeaderMap,
) -> Result<(), StatusCode> {
    match headers.get(TOKEN_HEADER) {
        Some(token)
            if token.to_str().map_err(|_| StatusCode::BAD_REQUEST)? == state.write_token => {}
        Some(token) => {
            log::warn!("Invalid token: {token:?}");
            return Err(StatusCode::UNAUTHORIZED);
        }
        _ => {
            log::warn!("Missing token");
            return Err(StatusCode::UNAUTHORIZED);
        }
    }

    let deadline_seconds = params
        .get("deadline_seconds")
        .and_then(|s| s.parse().ok())
        .unwrap_or(15usize);

    log::info!("Ping from {name}, deadline = {deadline_seconds}s");

    let mut services = state.services.lock().expect("To lock");
    let deadline = Instant::now() + Duration::from_secs(deadline_seconds as u64);
    match services.binary_search_by_key(&&name, |s| &s.name) {
        Ok(index) => services[index].deadline = deadline,
        Err(index) => {
            services.insert(index, ServiceState { name, deadline });
        }
    }

    Ok(())
}

async fn check(
    State(state): State<Arc<AppState>>,
    Path(name): Path<String>,
) -> Result<(), StatusCode> {
    let mut services = state.services.lock().expect("To lock");
    match services.binary_search_by_key(&&name, |s| &s.name) {
        Ok(index) => {
            if services[index].deadline < Instant::now() {
                log::info!("Service {name} has expired");
                services.remove(index);
                Err(StatusCode::GONE)
            } else {
                log::info!("Service {name} has not expired");
                Ok(())
            }
        }
        Err(_) => {
            log::info!("Service {name} has not been pinged or has expired");
            Err(StatusCode::GONE)
        }
    }
}

#[derive(Parser)]
struct Args {
    /// The address to listen on
    #[clap(short, long, default_value = "127.0.0.1:3400", env = "LISTEN_ADDR")]
    listen_addr: SocketAddr,

    /// The token to use for write operations
    #[clap(short, long, env = "TOKEN")]
    token: String,
}

#[tokio::main]
async fn main() {
    let _ = dotenvy::dotenv();
    env_logger::init();

    let Args { listen_addr, token } = Args::parse();

    let listener = TcpListener::bind(listen_addr).await.expect("To bind");
    log::info!("Listening on {listen_addr}");

    let app = Router::new()
        .route("/health/:name", axum::routing::put(ping))
        .route("/health/:name", axum::routing::get(check))
        .with_state(Arc::new(AppState {
            write_token: token,
            services: Mutex::new(Vec::new()),
        }));

    axum::serve(listener, app).await.unwrap();
}
