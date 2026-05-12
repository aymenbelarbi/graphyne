use tracing::{info, warn, error, debug, instrument};
use tracing_subscriber::{fmt, EnvFilter, layer::SubscriberExt, util::SubscriberInitExt, Layer};
use std::io;

pub fn init_logging(level: &str) -> Result<(), Box<dyn std::error::Error>> {
    let filter = EnvFilter::try_from_default_env()
        .unwrap_or_else(|_| EnvFilter::new(level));
    
    tracing_subscriber::registry()
        .with(
            fmt::layer()
                .with_target(false)
                .with_thread_ids(true)
                .with_file(true)
                .with_line_number(true)
                .with_filter(filter.clone())
        )
        .with(
            fmt::layer()
                .with_writer(io::stdout)
                .with_target(false)
                .with_thread_ids(true)
                .with_file(true)
                .with_line_number(true)
                .with_filter(filter)
        )
        .try_init()?;
    
    Ok(())
}

// Structured logging macros for Graphyne components
#[macro_export]
macro_rules! log_search {
    ($query:expr, $results:expr, $duration:expr) => {
        info!(
            target: "graphyne::search",
            query = %$query,
            results = %$results,
            duration_ms = %$duration,
            "Search completed"
        );
    };
}

#[macro_export]
macro_rules! log_memory_store {
    ($memory_id:expr, $space:expr) => {
        info!(
            target: "graphyne::memory",
            memory_id = %$memory_id,
            space = %$space,
            "Memory stored"
        );
    };
    ($memory_id:expr, $space:expr, $importance:expr) => {
        info!(
            target: "graphyne::memory",
            memory_id = %$memory_id,
            space = %$space,
            importance = %$importance,
            "Memory stored"
        );
    };
}

#[macro_export]
macro_rules! log_memory_recall {
    ($query:expr, $count:expr) => {
        info!(
            target: "graphyne::memory",
            query = %$query,
            results = %$count,
            "Memory recalled"
        );
    };
}

#[macro_export]
macro_rules! log_storage_operation {
    ($operation:expr, $success:expr) => {
        info!(
            target: "graphyne::storage",
            operation = %$operation,
            success = %$success,
            "Storage operation"
        );
    };
}

#[macro_export]
macro_rules! log_grpc_request {
    ($method:expr, $duration:expr) => {
        info!(
            target: "graphyne::grpc",
            method = %$method,
            duration_ms = %$duration,
            "gRPC request"
        );
    };
}

#[macro_export]
macro_rules! log_http_request {
    ($method:expr, $path:expr, $status:expr, $duration:expr) => {
        info!(
            target: "graphyne::http",
            method = %$method,
            path = %$path,
            status = %$status,
            duration_ms = %$duration,
            "HTTP request"
        );
    };
}

#[macro_export]
macro_rules! log_error {
    ($message:expr) => {
        error!(
            target: "graphyne::error",
            message = %$message,
            "Error occurred"
        );
    };
    ($message:expr, $error:expr) => {
        error!(
            target: "graphyne::error",
            message = %$message,
            error = %$error,
            "Error occurred"
        );
    };
}

#[macro_export]
macro_rules! log_warning {
    ($message:expr) => {
        warn!(
            target: "graphyne::warning",
            message = %$message,
            "Warning"
        );
    };
}
