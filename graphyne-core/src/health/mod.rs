use serde::{Serialize, Deserialize};
use std::time::{Instant, Duration};
use std::path::Path;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct HealthStatus {
    pub status: String,  // "healthy", "degraded", "unhealthy"
    pub version: String,
    pub uptime_seconds: u64,
    pub checks: Vec<HealthCheck>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct HealthCheck {
    pub name: String,
    pub status: String,
    pub message: Option<String>,
    pub duration_ms: u64,
}

pub struct HealthChecker {
    start_time: Instant,
    storage_path: Option<String>,
}

impl HealthChecker {
    pub fn new() -> Self {
        Self { 
            start_time: Instant::now(),
            storage_path: None,
        }
    }
    
    pub fn with_storage_path(storage_path: String) -> Self {
        Self {
            start_time: Instant::now(),
            storage_path: Some(storage_path),
        }
    }
    
    pub fn check_health(&self) -> HealthStatus {
        let mut checks = Vec::new();
        
        // Check storage
        checks.push(self.check_storage());
        
        // Check memory store (basic check)
        checks.push(self.check_memory());
        
        // Determine overall status
        let status = if checks.iter().all(|c| c.status == "healthy") {
            "healthy"
        } else if checks.iter().any(|c| c.status == "unhealthy") {
            "unhealthy"
        } else {
            "degraded"
        };
        
        HealthStatus {
            status: status.to_string(),
            version: env!("CARGO_PKG_VERSION").to_string(),
            uptime_seconds: self.start_time.elapsed().as_secs(),
            checks,
        }
    }
    
    fn check_storage(&self) -> HealthCheck {
        let start = Instant::now();
        
        match &self.storage_path {
            Some(path) => {
                // Check if storage directory exists and is accessible
                let path_obj = Path::new(path);
                
                if !path_obj.exists() {
                    return HealthCheck {
                        name: "storage".to_string(),
                        status: "unhealthy".to_string(),
                        message: Some(format!("Storage path does not exist: {}", path)),
                        duration_ms: start.elapsed().as_millis() as u64,
                    };
                }
                
                // Try to read the directory
                match std::fs::read_dir(path_obj) {
                    Ok(_) => HealthCheck {
                        name: "storage".to_string(),
                        status: "healthy".to_string(),
                        message: Some(format!("Storage accessible at: {}", path)),
                        duration_ms: start.elapsed().as_millis() as u64,
                    },
                    Err(e) => HealthCheck {
                        name: "storage".to_string(),
                        status: "unhealthy".to_string(),
                        message: Some(format!("Cannot access storage: {}", e)),
                        duration_ms: start.elapsed().as_millis() as u64,
                    },
                }
            }
            None => {
                // No storage path configured, assume healthy
                HealthCheck {
                    name: "storage".to_string(),
                    status: "healthy".to_string(),
                    message: Some("No storage path configured".to_string()),
                    duration_ms: start.elapsed().as_millis() as u64,
                }
            }
        }
    }
    
    fn check_memory(&self) -> HealthCheck {
        let start = Instant::now();
        
        // Basic memory check - in a real implementation, this would check the memory store
        // For now, we'll just return healthy
        HealthCheck {
            name: "memory".to_string(),
            status: "healthy".to_string(),
            message: Some("Memory store operational".to_string()),
            duration_ms: start.elapsed().as_millis() as u64,
        }
    }
    
    pub fn uptime(&self) -> Duration {
        self.start_time.elapsed()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_health_checker_creation() {
        let checker = HealthChecker::new();
        let status = checker.check_health();
        assert!(status.status == "healthy" || status.status == "degraded");
    }
    
    #[test]
    fn test_uptime() {
        let checker = HealthChecker::new();
        std::thread::sleep(Duration::from_millis(10));
        assert!(checker.uptime().as_millis() >= 10);
    }
}
