use prometheus::{Encoder, Gauge, Counter, Histogram, Registry, HistogramOpts, Opts};
use std::sync::Arc;
use std::time::Instant;

pub struct GraphyneMetrics {
    pub registry: Registry,
    // Search metrics
    pub search_requests_total: Counter,
    pub search_duration_seconds: Histogram,
    pub search_results_count: Histogram,
    // Memory metrics
    pub memory_stored_total: Counter,
    pub memory_recalled_total: Counter,
    pub memory_entries_current: Gauge,
    // Storage metrics
    pub storage_operations_total: Counter,
    pub storage_operation_duration: Histogram,
    // API metrics
    pub grpc_requests_total: Counter,
    pub http_requests_total: Counter,
}

impl GraphyneMetrics {
    pub fn new() -> Result<Self, prometheus::Error> {
        let registry = Registry::new();
        
        // Search metrics
        let search_requests_total = Counter::with_opts(
            Opts::new("graphyne_search_requests_total", "Total number of search requests")
        )?;
        registry.register(Box::new(search_requests_total.clone()))?;
        
        let search_duration_seconds = Histogram::with_opts(
            HistogramOpts::new("graphyne_search_duration_seconds", "Search duration in seconds")
                .buckets(vec![0.001, 0.005, 0.01, 0.025, 0.05, 0.1, 0.25, 0.5, 1.0, 2.5, 5.0, 10.0])
        )?;
        registry.register(Box::new(search_duration_seconds.clone()))?;
        
        let search_results_count = Histogram::with_opts(
            HistogramOpts::new("graphyne_search_results_count", "Number of search results returned")
                .buckets(vec![0.0, 1.0, 5.0, 10.0, 25.0, 50.0, 100.0, 250.0, 500.0, 1000.0])
        )?;
        registry.register(Box::new(search_results_count.clone()))?;
        
        // Memory metrics
        let memory_stored_total = Counter::with_opts(
            Opts::new("graphyne_memory_stored_total", "Total number of memories stored")
        )?;
        registry.register(Box::new(memory_stored_total.clone()))?;
        
        let memory_recalled_total = Counter::with_opts(
            Opts::new("graphyne_memory_recalled_total", "Total number of memories recalled")
        )?;
        registry.register(Box::new(memory_recalled_total.clone()))?;
        
        let memory_entries_current = Gauge::with_opts(
            Opts::new("graphyne_memory_entries_current", "Current number of memory entries")
        )?;
        registry.register(Box::new(memory_entries_current.clone()))?;
        
        // Storage metrics
        let storage_operations_total = Counter::with_opts(
            Opts::new("graphyne_storage_operations_total", "Total number of storage operations")
        )?;
        registry.register(Box::new(storage_operations_total.clone()))?;
        
        let storage_operation_duration = Histogram::with_opts(
            HistogramOpts::new("graphyne_storage_operation_duration_seconds", "Storage operation duration in seconds")
                .buckets(vec![0.0001, 0.0005, 0.001, 0.005, 0.01, 0.05, 0.1, 0.5, 1.0])
        )?;
        registry.register(Box::new(storage_operation_duration.clone()))?;
        
        // API metrics
        let grpc_requests_total = Counter::with_opts(
            Opts::new("graphyne_grpc_requests_total", "Total number of gRPC requests")
        )?;
        registry.register(Box::new(grpc_requests_total.clone()))?;
        
        let http_requests_total = Counter::with_opts(
            Opts::new("graphyne_http_requests_total", "Total number of HTTP requests")
        )?;
        registry.register(Box::new(http_requests_total.clone()))?;
        
        Ok(Self {
            registry,
            search_requests_total,
            search_duration_seconds,
            search_results_count,
            memory_stored_total,
            memory_recalled_total,
            memory_entries_current,
            storage_operations_total,
            storage_operation_duration,
            grpc_requests_total,
            http_requests_total,
        })
    }
    
    pub fn export(&self) -> Result<String, prometheus::Error> {
        let encoder = prometheus::TextEncoder::new();
        let metric_families = self.registry.gather();
        let mut buffer = Vec::new();
        encoder.encode(&metric_families, &mut buffer)?;
        Ok(String::from_utf8(buffer).map_err(|e| prometheus::Error::Msg(e.to_string()))?)
    }
}

/// Helper struct to measure duration of operations
pub struct OperationTimer {
    start: Instant,
    histogram: Histogram,
}

impl OperationTimer {
    pub fn new(histogram: &Histogram) -> Self {
        Self {
            start: Instant::now(),
            histogram: histogram.clone(),
        }
    }
}

impl Drop for OperationTimer {
    fn drop(&mut self) {
        let duration = self.start.elapsed().as_secs_f64();
        self.histogram.observe(duration);
    }
}

/// Macro to time an operation and record to histogram
#[macro_export]
macro_rules! time_operation {
    ($histogram:expr, $block:block) => {{
        let timer = OperationTimer::new(&$histogram);
        let result = $block;
        drop(timer);
        result
    }};
}

pub type Result<T> = std::result::Result<T, prometheus::Error>;
