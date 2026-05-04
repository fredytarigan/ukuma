use axum::body::Bytes;
use axum::http::{Response, StatusCode, header};
use http_body_util::Full;
use std::any::Any;
use tokio::signal;

pub fn print_welcome() {
    let welcome: &'static str = r#"
   ▗▖ ▗▖▗▄▄▄▖▗▄▄▖ ▗▄▄▄▖     ▗▄▄▖ ▗▄▖ ▗▖  ▗▖▗▄▄▄▖ ▗▄▄▖     ▗▄▖     ▗▄▄▖ ▗▄▄▄▖ ▗▄▄▖    ▗▄▄▄ ▗▄▄▖ ▗▄▄▄▖ ▗▄▖ ▗▖  ▗▖
   ▐▌ ▐▌▐▌   ▐▌ ▐▌▐▌       ▐▌   ▐▌ ▐▌▐▛▚▞▜▌▐▌   ▐▌       ▐▌ ▐▌    ▐▌ ▐▌  █  ▐▌       ▐▌  █▐▌ ▐▌▐▌   ▐▌ ▐▌▐▛▚▞▜▌
   ▐▛▀▜▌▐▛▀▀▘▐▛▀▚▖▐▛▀▀▘    ▐▌   ▐▌ ▐▌▐▌  ▐▌▐▛▀▀▘ ▝▀▚▖    ▐▛▀▜▌    ▐▛▀▚▖  █  ▐▌▝▜▌    ▐▌  █▐▛▀▚▖▐▛▀▀▘▐▛▀▜▌▐▌  ▐▌
   ▐▌ ▐▌▐▙▄▄▖▐▌ ▐▌▐▙▄▄▖    ▝▚▄▄▖▝▚▄▞▘▐▌  ▐▌▐▙▄▄▖▗▄▄▞▘    ▐▌ ▐▌    ▐▙▄▞▘▗▄█▄▖▝▚▄▞▘    ▐▙▄▄▀▐▌ ▐▌▐▙▄▄▖▐▌ ▐▌▐▌  ▐▌

   =============================================================================================================
   
                                           *** ad maiora natus sum ***
   "#;

    println!("{}", welcome);
}

pub fn handle_panic(err: Box<dyn Any + Send + 'static>) -> Response<Full<Bytes>> {
    let details = if let Some(s) = err.downcast_ref::<String>() {
        s.clone()
    } else if let Some(s) = err.downcast_ref::<&str>() {
        s.to_string()
    } else {
        "Unknown panic message".to_string()
    };

    let body = serde_json::json!({
        "error": {
            "kind": "panic",
            "details": details,
        }
    });
    let body = serde_json::to_string(&body).unwrap();

    Response::builder()
        .status(StatusCode::INTERNAL_SERVER_ERROR)
        .header(header::CONTENT_TYPE, "application/json")
        .body(Full::from(body))
        .unwrap()
}

pub async fn shutdown() {
    let ctrl_c = async {
        signal::ctrl_c()
            .await
            .expect("failed to setup ctrl+c handler");
    };

    #[cfg(unix)]
    let terminate = async {
        signal::unix::signal(signal::unix::SignalKind::terminate())
            .expect("failed to install signal handler")
            .recv()
            .await;
    };

    #[cfg(not(unix))]
    let terminate = std::future::pending::<()>();

    tokio::select! {
        _ = ctrl_c => {},
        _ = terminate => {},
    }
}
