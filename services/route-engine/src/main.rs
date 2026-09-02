mod health;

pub mod proto {
    tonic::include_proto!("route.v1");
}

use std::{env, net::SocketAddr, time::Duration};

use health::HealthService;
use proto::route_optimizer_server::RouteOptimizerServer;
use tonic::transport::Server;
use url::Url;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let listen_address: SocketAddr = env_or_default("ROUTE_ENGINE_LISTEN_ADDRESS", "0.0.0.0:50051")
        .parse()
        .map_err(|error| format!("invalid ROUTE_ENGINE_LISTEN_ADDRESS: {error}"))?;
    let graphhopper_url = Url::parse(&env_or_default(
        "GRAPHHOPPER_URL",
        "http://graphhopper:8989/",
    ))?;
    let health_service = HealthService::new(graphhopper_url, Duration::from_secs(2))?;

    println!("route engine listening on {listen_address}");
    Server::builder()
        .add_service(RouteOptimizerServer::new(health_service))
        .serve_with_shutdown(listen_address, shutdown_signal())
        .await?;
    Ok(())
}

fn env_or_default(name: &str, fallback: &str) -> String {
    env::var(name).unwrap_or_else(|_| fallback.to_owned())
}

async fn shutdown_signal() {
    let ctrl_c = async {
        tokio::signal::ctrl_c()
            .await
            .expect("failed to install Ctrl+C handler");
    };

    #[cfg(unix)]
    let terminate = async {
        tokio::signal::unix::signal(tokio::signal::unix::SignalKind::terminate())
            .expect("failed to install SIGTERM handler")
            .recv()
            .await;
    };

    #[cfg(not(unix))]
    let terminate = std::future::pending::<()>();

    tokio::select! {
        () = ctrl_c => {},
        () = terminate => {},
    }
}
