//! Graphyne Server - Main entry point
//!
//! Starts both gRPC and HTTP servers for Graphyne services.

use std::net::SocketAddr;
use std::sync::Arc;
use tokio::signal;
use tracing::{info, error};

mod grpc;
mod http;

use grpc::{
    SearchServiceImpl, DocumentServiceImpl, VectorServiceImpl,
    GraphServiceImpl, MemoryServiceImpl, AdminServiceImpl,
};
use graphyne_proto::graphyne::{
    search_service_server::SearchServiceServer,
    document_service_server::DocumentServiceServer,
    vector_service_server::VectorServiceServer,
    graph_service_server::GraphServiceServer,
    memory_service_server::MemoryServiceServer,
    admin_service_server::AdminServiceServer,
};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize logging using graphyne-core
    graphyne_core::logging::init_logging("info")?;
    
    info!("Starting Graphyne Server...");
    
    // Create metrics and health checker
    let metrics = Arc::new(graphyne_core::metrics::GraphyneMetrics::new()?);
    let health_checker = graphyne_core::health::HealthChecker::new();
    let admin_service = graphyne_core::admin::AdminService::new(
        Arc::clone(&metrics),
        health_checker.clone(),
    );
    
    // Create service implementations
    let search_service = SearchServiceImpl::new();
    let document_service = DocumentServiceImpl::new();
    let vector_service = VectorServiceImpl::new();
    let graph_service = GraphServiceImpl::new();
    let memory_service = MemoryServiceImpl::new();
    let admin_grpc_service = AdminServiceImpl::new(
        Arc::clone(&metrics),
        health_checker.clone(),
        admin_service,
    );
    
    // Define addresses
    let grpc_addr: SocketAddr = "0.0.0.0:50051".parse()?;
    let http_addr: SocketAddr = "0.0.0.0:8080".parse()?;
    
    // Create HTTP app state
    let http_state = http::AppState {
        memory_store: Arc::new(tokio::sync::Mutex::new(
            graphyne_core::memory::MemoryStore::new()
        )),
        metrics: Arc::clone(&metrics),
        health_checker: Arc::new(health_checker),
        admin_service: Arc::new(tokio::sync::Mutex::new(
            graphyne_core::admin::AdminService::new(
                Arc::clone(&metrics),
                graphyne_core::health::HealthChecker::new(),
            )
        )),
    };
    
    // Create gRPC server
    let grpc_server = tonic::transport::Server::builder()
        .add_service(SearchServiceServer::new(search_service))
        .add_service(DocumentServiceServer::new(document_service))
        .add_service(VectorServiceServer::new(vector_service))
        .add_service(GraphServiceServer::new(graph_service))
        .add_service(MemoryServiceServer::new(memory_service))
        .add_service(AdminServiceServer::new(admin_grpc_service))
        .serve(grpc_addr);
    
    info!("gRPC server listening on {}", grpc_addr);
    
    // Create HTTP server
    let http_server = http::start_http_server(http_addr, http_state);
    
    info!("HTTP server listening on {}", http_addr);
    
    // Run both servers concurrently
    tokio::select! {
        result = grpc_server => {
            match result {
                Ok(_) => info!("gRPC server stopped"),
                Err(e) => error!("gRPC server error: {}", e),
            }
        }
        result = http_server => {
            match result {
                Ok(_) => info!("HTTP server stopped"),
                Err(e) => error!("HTTP server error: {}", e),
            }
        }
        _ = shutdown_signal() => {
            info!("Shutdown signal received");
        }
    }
    
    info!("Graphyne Server stopped");
    Ok(())
}

/// Wait for shutdown signal (Ctrl+C or SIGTERM)
async fn shutdown_signal() {
    #[cfg(unix)]
    {
        use tokio::signal::unix::{signal, SignalKind};
        
        let mut sigint = signal(SignalKind::interrupt()).expect("Failed to install SIGINT handler");
        let mut sigterm = signal(SignalKind::terminate()).expect("Failed to install SIGTERM handler");
        
        tokio::select! {
            _ = sigint.recv() => {
                info!("Received SIGINT");
            }
            _ = sigterm.recv() => {
                info!("Received SIGTERM");
            }
        }
    }
    
    #[cfg(not(unix))]
    {
        signal::ctrl_c()
            .await
            .expect("Failed to install Ctrl+C handler");
        info!("Received Ctrl+C");
    }
}
