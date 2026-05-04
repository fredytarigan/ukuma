use anyhow::Result;
use axum::{
    Router, ServiceExt,
    extract::Request,
    http::{HeaderMap, HeaderValue, StatusCode},
    routing::{get},
};
use std::{net::SocketAddr, sync::Arc, time::Duration};
use tower_default_headers::DefaultHeadersLayer;
use tower_http::{
    catch_panic::CatchPanicLayer,
    timeout::TimeoutLayer,
    trace::{self, TraceLayer},
};
use tracing::{info, Level};
use tokio::net::TcpListener;

use cmd::helpers;

#[tokio::main]
async fn main() -> Result<()> {
    // print welcome message
    helpers::print_welcome();

    let current_dir = std::env::current_dir()?;

    // initializing application configuration
    println!("initializing application configuration");
    let config_path = match dotenvy::var("UKUMA_CONFIG_PATH") {
        Ok(path) => path,
        Err(_) => current_dir.join("config/config.yaml").to_string_lossy().to_string(),
    };
    let config = Arc::new(app_config::read_from_path(config_path)?);

    if config.log.level == "debug" {
        println!("Config: {:?}", config);
    };

    // initializing application logging level
    let level = match config.log.level.as_str() {
        "debug" => Level::DEBUG,
        "info" => Level::INFO,
        "warn" => Level::WARN,
        "error" => Level::ERROR,
        _ => Level::INFO,
    };

    // initializing application logger
    println!("initializing application logger");
    let logger = app_logger::Logger { level };
    logger.setup()?;
    info!("application logger initialized successfully");

    let worker = workers::spawn(config.clone());
    info!(
        interval_secs = config.agent.interval,
        checks = config.checks.len(),
        "check worker started"
    );

    // set default response header
    let mut default_headers = HeaderMap::new();
    default_headers.insert("X-Server", HeaderValue::from_static("Ukuma"));
    default_headers.insert("X-Version", HeaderValue::from_static(env!("CARGO_PKG_VERSION")));

    // setup axum http server
    let app = Router::new()
        .layer(DefaultHeadersLayer::new(default_headers))
        .layer(TimeoutLayer::with_status_code(
            StatusCode::REQUEST_TIMEOUT,
            Duration::from_secs(10),
        ))
        .layer(
            TraceLayer::new_for_http()
                .make_span_with(trace::DefaultMakeSpan::new().level(level))
                .on_response(trace::DefaultOnResponse::new().level(level)),
        )
        .layer(CatchPanicLayer::custom(helpers::handle_panic))
        .route("/", get(root_handler))
        .route("/healthz", get(healthz_handler));

    // configure http listener
    let listener = TcpListener::bind("0.0.0.0:8115").await.unwrap();


    // start http server
    info!("starting http server, running on address 0.0.0.0:8115");
    axum::serve(
        listener,
        ServiceExt::<Request>::into_make_service_with_connect_info::<SocketAddr>(app),
    )
    .with_graceful_shutdown(helpers::shutdown())
    .await
    .unwrap();

    worker.abort();
    info!("check worker stopped");

    Ok(())
}

async fn root_handler() -> &'static str {
    "Hello, World!"
}

async fn healthz_handler() -> &'static str {
    "OK"
}