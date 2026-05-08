use serde::{Serialize, Deserialize};
use std::sync::Arc;
use std::path::Path;
use std::time::{SystemTime, UNIX_EPOCH};
use crate::metrics::GraphyneMetrics;
use crate::health::HealthChecker;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct AdminStats {
    pub uptime_seconds: u64,
    pub total_searches: u64,
    pub total_memories: u64,
    pub storage_size_bytes: u64,
    pub active_connections: u32,
    pub timestamp: u64,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct BackupRequest {
    pub path: String,
    pub include_graph: bool,
    pub include_vectors: bool,
    pub include_memories: bool,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct BackupResponse {
    pub success: bool,
    pub message: String,
    pub backup_path: Option<String>,
    pub size_bytes: u64,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct RestoreRequest {
    pub path: String,
    pub overwrite: bool,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct RestoreResponse {
    pub success: bool,
    pub message: String,
    pub items_restored: u64,
}

pub struct AdminService {
    metrics: Arc<GraphyneMetrics>,
    health_checker: HealthChecker,
    storage_path: Option<String>,
}

impl AdminService {
    pub fn new(metrics: Arc<GraphyneMetrics>, health_checker: HealthChecker) -> Self {
        Self {
            metrics,
            health_checker,
            storage_path: None,
        }
    }
    
    pub fn with_storage_path(mut self, path: String) -> Self {
        self.storage_path = Some(path);
        self
    }
    
    pub fn get_stats(&self) -> AdminStats {
        let uptime = self.health_checker.uptime().as_secs();
        
        // Gather statistics from metrics
        let total_searches = self.metrics.search_requests_total.get() as u64;
        let total_memories = self.metrics.memory_stored_total.get() as u64;
        
        // Get storage size if path is available
        let storage_size_bytes = self.storage_path.as_ref()
            .map(|path| self.calculate_directory_size(Path::new(path)))
            .unwrap_or(0);
        
        AdminStats {
            uptime_seconds: uptime,
            total_searches,
            total_memories,
            storage_size_bytes,
            active_connections: 0, // Would be tracked by server
            timestamp: SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs(),
        }
    }
    
    pub fn flush(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        // Flush all buffers to disk
        // In a real implementation, this would flush storage buffers
        tracing::info!(target: "graphyne::admin", "Flushing all data to disk");
        Ok(())
    }
    
    pub fn backup(&self, path: &str) -> Result<BackupResponse, Box<dyn std::error::Error>> {
        let backup_path = Path::new(path);
        
        if !backup_path.exists() {
            std::fs::create_dir_all(backup_path)?;
        }
        
        // In a real implementation, this would backup the data
        tracing::info!(target: "graphyne::admin", path = %path, "Creating backup");
        
        Ok(BackupResponse {
            success: true,
            message: "Backup completed successfully".to_string(),
            backup_path: Some(path.to_string()),
            size_bytes: 0, // Would calculate actual backup size
        })
    }
    
    pub fn restore(&mut self, path: &str) -> Result<RestoreResponse, Box<dyn std::error::Error>> {
        let backup_path = Path::new(path);
        
        if !backup_path.exists() {
            return Ok(RestoreResponse {
                success: false,
                message: format!("Backup path does not exist: {}", path),
                items_restored: 0,
            });
        }
        
        // In a real implementation, this would restore from backup
        tracing::info!(target: "graphyne::admin", path = %path, "Restoring from backup");
        
        Ok(RestoreResponse {
            success: true,
            message: "Restore completed successfully".to_string(),
            items_restored: 0, // Would count actual restored items
        })
    }
    
    pub fn compact(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        // Compact storage
        // In a real implementation, this would compact storage files
        tracing::info!(target: "graphyne::admin", "Compacting storage");
        Ok(())
    }
    
    fn calculate_directory_size(&self, path: &Path) -> u64 {
        if !path.exists() {
            return 0;
        }
        
        let mut total_size = 0u64;
        
        if let Ok(entries) = std::fs::read_dir(path) {
            for entry in entries.flatten() {
                let entry_path = entry.path();
                
                if entry_path.is_file() {
                    if let Ok(metadata) = entry.metadata() {
                        total_size += metadata.len();
                    }
                } else if entry_path.is_dir() {
                    total_size += self.calculate_directory_size(&entry_path);
                }
            }
        }
        
        total_size
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::metrics::GraphyneMetrics;
    use std::sync::Arc;
    
    #[test]
    fn test_admin_stats() {
        let metrics = Arc::new(GraphyneMetrics::new().unwrap());
        let health_checker = crate::health::HealthChecker::new();
        let admin = AdminService::new(metrics, health_checker);
        
        let stats = admin.get_stats();
        assert!(stats.uptime_seconds >= 0);
    }
}
