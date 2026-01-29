mod proxy;
mod telemetry;

use std::net::SocketAddr;
use tokio::net::TcpListener;
use hyper::server::conn::http1;
use hyper_util::rt::TokioIo;
use hyper::service::service_fn;
use crate::proxy::ProxyService;
use tracing::info;

#[tokio::main] // Using standard Tokio runtime for MVP (Phase 1)
async fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    // 1. Initialize Telemetry
    telemetry::init_telemetry();
    
    // 2. Configuration (Hardcoded for MVP)
    let addr = SocketAddr::from(([0, 0, 0, 0], 3000));
    let proxy_service = ProxyService::new();

    // 3. Bind TCP Listener
    let listener = TcpListener::bind(addr).await?;
    info!("Hertz-L7LB listening on http://{}", addr);
    info!("Forwarding traffic to http://127.0.0.1:8080");

    // 4. Accept Loop
    loop {
        let (stream, remote_addr) = listener.accept().await?;
        
        // Use TokioIo adapter for Hyper v1
        let io = TokioIo::new(stream);
        let service = proxy_service.clone();

        // Spawn a task per connection (Tokio's green threads)
        tokio::task::spawn(async move {
            // Hyper v1 low-level server builder
            // We use http1 only for this MVP phase
            if let Err(err) = http1::Builder::new()
                .serve_connection(io, service_fn(move |req| {
                    let s = service.clone();
                    async move { s.handle_request(req).await }
                }))
                .await
            {
                tracing::error!("Error serving connection from {}: {:?}", remote_addr, err);
            }
        });
    }
}