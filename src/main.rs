use tonic::transport::Server;
use tracing::{debug, info};

use dynlog::fibonacci_server::FibonacciServer;
use dynlog::logging_server::LoggingServer;

pub mod dynlog {
    tonic::include_proto!("dynlog");
}

mod logging_api;
mod service_runner;

#[tokio::main]
async fn main() -> Result<(), String> {
    let log_handler = logging_api::DynamicLogHandler::new();

    debug!("We are about to start... but you won't see this line");
    info!("Hello, DevFest!");

    let addr = "0.0.0.0:50051".parse().unwrap();
    let service = service_runner::ServiceRunner::default();
    info!("Running gRPC server at: {}", addr);
    Server::builder()
        .add_service(LoggingServer::new(log_handler))
        .add_service(FibonacciServer::new(service))
        .serve(addr)
        .await
        .map_err(|e| e.to_string())?;

    Ok(())
}
