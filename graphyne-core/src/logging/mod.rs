use tracing::{info, warn, error, debug, instrument};
use tracing_subscriber::{fmt, EnvFilter, layer::SubscriberExt, util::SubscriberInitExt};
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
}

#[macro_export]
macro_rules! log_memory_recall {
    ($query:expr, $results:expr) => {
        info!(
            target: "graphyne::memory",
            query = %$query,
            results = %$results,
            "Memory recalled"
        );
    };
}

#[macro_export]
macro_rules! log_storage_operation {
    ($operation:expr, $collection:expr, $bucket:expr) => {
        debug!(
            target: "graphyne::storage",
            operation = %$operation,
            collection = %$collection,
            bucket = %$bucket,
            "Storage operation"
        );
    };
}

#[macro_export]
macro_rules! log_grpc_request {
    ($method:expr, $request_id:expr) => {
        info!(
            target: "graphyne::grpc",
            method = %$method,
            request_id = %$request_id,
            "gRPC request received"
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
            "HTTP request completed"
        );
    };
}

#[macro_export]
macro_rules! log_error {
    ($error:expr, $context:expr) => {
        error!(
            target: "graphyne::error",
            error = %$error,
            context = %$context,
            "Error occurred"
        );
    };
}

#[macro_export]
macro_rules! log_warning {
    ($message:expr, $context:expr) => {
        warn!(
            target: "graphyne::warning",
            message = %$message,
            context = %$context,
            "Warning"
        );
    };
}

/// Span attribute helper for tracing operations
pub fn trace_search<F, R>(query: &str, f: F) -> R
where
    F: FnOnce() -> R,
{
    let span = tracing::info_span!("search", query = %query);
    let _enter = span.enter();
    f()
}

pub fn trace_memory_operation<F, R>(operation: &str, memory_id: &str, f: F) -> R
where
    F: FnOnce() -> R,
{
    let span = tracing::info_span!("memory", operation = %operation, memory_id = %memory_id);
    let _enter = span.enter();
    f()
}

pub fn trace_storage_operation<F, R>(operation: &str, collection: &str, f: F) -> R
where
    F: FnOnce() -> R,
{
    let span = tracing::debug_span!("storage", operation = %operation, collection = %collection);
    let _enter = span.enter();
    f()
}
