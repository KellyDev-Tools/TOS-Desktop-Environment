//! Phase 16: Container Metrics
//!
//! Resource usage monitoring and metrics collection for containers.

use super::{ContainerId, ContainerResult, ContainerError};
use std::collections::HashMap;
use serde::{Deserialize, Serialize};

/// Container metrics
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ContainerMetrics {
    /// Container ID
    pub container_id: ContainerId,
    /// Timestamp
    pub timestamp: chrono::DateTime<chrono::Utc>,
    /// CPU metrics
    pub cpu: CpuMetrics,
    /// Memory metrics
    pub memory: MemoryMetrics,
    /// Network metrics
    pub network: NetworkMetrics,
    /// Block I/O metrics
    pub block_io: BlockIoMetrics,
    /// PIDs metrics
    pub pids: PidsMetrics,
}

/// CPU metrics
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct CpuMetrics {
    /// CPU usage percentage (0-100 per core, can exceed 100 for multi-core)
    pub usage_percent: f64,
    /// CPU usage in nanoseconds
    pub usage_nanoseconds: u64,
    /// System CPU usage in nanoseconds
    pub system_nanoseconds: u64,
    /// Number of periods throttled
    pub throttled_periods: u64,
    /// Throttled time in nanoseconds
    pub throttled_nanoseconds: u64,
    /// Number of CPU cores
    pub cpu_count: u32,
    /// Per-core usage
    pub per_cpu_usage: Vec<u64>,
}

/// Memory metrics
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct MemoryMetrics {
    /// Current memory usage in bytes
    pub usage_bytes: u64,
    /// Maximum memory usage in bytes
    pub max_usage_bytes: u64,
    /// Memory limit in bytes
    pub limit_bytes: u64,
    /// Memory usage percentage
    pub usage_percent: f64,
    /// Active anonymous memory
    pub active_anon_bytes: u64,
    /// Active file memory
    pub active_file_bytes: u64,
    /// Inactive anonymous memory
    pub inactive_anon_bytes: u64,
    /// Inactive file memory
    pub inactive_file_bytes: u64,
    /// Unevictable memory
    pub unevictable_bytes: u64,
    /// Memory + swap usage
    pub swap_usage_bytes: u64,
    /// Swap limit
    pub swap_limit_bytes: u64,
    /// Page faults
    pub page_faults: u64,
    /// Major page faults
    pub major_page_faults: u64,
    /// Kernel memory usage
    pub kernel_usage_bytes: u64,
    /// Kernel memory limit
    pub kernel_limit_bytes: u64,
}

/// Network metrics
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct NetworkMetrics {
    /// Total received bytes
    pub rx_bytes: u64,
    /// Total transmitted bytes
    pub tx_bytes: u64,
    /// Received packets
    pub rx_packets: u64,
    /// Transmitted packets
    pub tx_packets: u64,
    /// Receive errors
    pub rx_errors: u64,
    /// Transmit errors
    pub tx_errors: u64,
    /// Receive dropped
    pub rx_dropped: u64,
    /// Transmit dropped
    pub tx_dropped: u64,
    /// Per-interface metrics
    pub interfaces: HashMap<String, InterfaceMetrics>,
}

/// Interface-specific metrics
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct InterfaceMetrics {
    /// Interface name
    pub name: String,
    /// Received bytes
    pub rx_bytes: u64,
    /// Transmitted bytes
    pub tx_bytes: u64,
    /// Received packets
    pub rx_packets: u64,
    /// Transmitted packets
    pub tx_packets: u64,
    /// Receive errors
    pub rx_errors: u64,
    /// Transmit errors
    pub tx_errors: u64,
    /// Receive dropped
    pub rx_dropped: u64,
    /// Transmit dropped
    pub tx_dropped: u64,
}

/// Block I/O metrics
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct BlockIoMetrics {
    /// I/O service bytes (read)
    pub io_service_bytes_read: u64,
    /// I/O service bytes (write)
    pub io_service_bytes_write: u64,
    /// I/O serviced (read operations)
    pub io_serviced_read: u64,
    /// I/O serviced (write operations)
    pub io_serviced_write: u64,
    /// I/O service time (read) in nanoseconds
    pub io_service_time_read: u64,
    /// I/O service time (write) in nanoseconds
    pub io_service_time_write: u64,
    /// I/O wait time (read) in nanoseconds
    pub io_wait_time_read: u64,
    /// I/O wait time (write) in nanoseconds
    pub io_wait_time_write: u64,
    /// I/O merged (read)
    pub io_merged_read: u64,
    /// I/O merged (write)
    pub io_merged_write: u64,
    /// I/O time (read) in nanoseconds
    pub io_time_read: u64,
    /// I/O time (write) in nanoseconds
    pub io_time_write: u64,
    /// Sectors read
    pub sectors_read: u64,
    /// Sectors written
    pub sectors_write: u64,
    /// Per-device metrics
    pub devices: Vec<DeviceIoMetrics>,
}

/// Per-device I/O metrics
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct DeviceIoMetrics {
    /// Device major number
    pub major: u64,
    /// Device minor number
    pub minor: u64,
    /// Device name
    pub device_name: String,
    /// Read bytes
    pub read_bytes: u64,
    /// Write bytes
    pub write_bytes: u64,
    /// Read operations
    pub read_ops: u64,
    /// Write operations
    pub write_ops: u64,
    /// Read time in nanoseconds
    pub read_time: u64,
    /// Write time in nanoseconds
    pub write_time: u64,
}

/// PIDs metrics
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct PidsMetrics {
    /// Current number of PIDs
    pub current: u64,
    /// Maximum number of PIDs
    pub limit: u64,
    /// PID usage percentage
    pub usage_percent: f64,
}

/// Resource usage summary
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceUsage {
    /// CPU cores used
    pub cpu_cores: f64,
    /// Memory used in bytes
    pub memory_bytes: u64,
    /// Memory limit in bytes
    pub memory_limit_bytes: u64,
    /// Network received bytes
    pub network_rx_bytes: u64,
    /// Network transmitted bytes
    pub network_tx_bytes: u64,
    /// Block read bytes
    pub block_read_bytes: u64,
    /// Block write bytes
    pub block_write_bytes: u64,
    /// PIDs used
    pub pids: u64,
}

impl ResourceUsage {
    /// Calculate total resource usage from metrics
    pub fn from_metrics(metrics: &ContainerMetrics) -> Self {
        Self {
            cpu_cores: metrics.cpu.usage_percent / 100.0,
            memory_bytes: metrics.memory.usage_bytes,
            memory_limit_bytes: metrics.memory.limit_bytes,
            network_rx_bytes: metrics.network.rx_bytes,
            network_tx_bytes: metrics.network.tx_bytes,
            block_read_bytes: metrics.block_io.io_service_bytes_read,
            block_write_bytes: metrics.block_io.io_service_bytes_write,
            pids: metrics.pids.current,
        }
    }
    
    /// Get memory usage percentage
    pub fn memory_percent(&self) -> f64 {
        if self.memory_limit_bytes == 0 {
            0.0
        } else {
            (self.memory_bytes as f64 / self.memory_limit_bytes as f64) * 100.0
        }
    }
    
    /// Format memory for display
    pub fn format_memory(&self) -> String {
        format_bytes(self.memory_bytes)
    }
    
    /// Format memory limit for display
    pub fn format_memory_limit(&self) -> String {
        format_bytes(self.memory_limit_bytes)
    }
}

/// Metrics collector for containers
#[derive(Debug)]
pub struct MetricsCollector {
    history: std::sync::Mutex<HashMap<ContainerId, Vec<ContainerMetrics>>>,
    max_history: usize,
}

impl MetricsCollector {
    /// Create a new metrics collector
    pub fn new() -> Self {
        Self {
            history: std::sync::Mutex::new(HashMap::new()),
            max_history: 100,
        }
    }
    
    /// Create with custom history size
    pub fn with_history(max_history: usize) -> Self {
        Self {
            history: std::sync::Mutex::new(HashMap::new()),
            max_history,
        }
    }
    
    /// Record metrics for a container
    pub fn record(&self, metrics: ContainerMetrics) {
        let mut history = self.history.lock().unwrap();
        let entries = history.entry(metrics.container_id.clone()).or_insert_with(Vec::new);
        
        entries.push(metrics);
        
        // Trim history if needed
        if entries.len() > self.max_history {
            entries.remove(0);
        }
    }
    
    /// Get latest metrics for a container
    pub fn get_latest(&self, container_id: &ContainerId) -> Option<ContainerMetrics> {
        let history = self.history.lock().unwrap();
        history.get(container_id)?.last().cloned()
    }
    
    /// Get metrics history for a container
    pub fn get_history(&self, container_id: &ContainerId) -> Vec<ContainerMetrics> {
        let history = self.history.lock().unwrap();
        history.get(container_id).cloned().unwrap_or_default()
    }
    
    /// Get average CPU usage over time period
    pub fn get_average_cpu(&self, container_id: &ContainerId, duration: std::time::Duration) -> Option<f64> {
        let history = self.get_history(container_id);
        let cutoff = chrono::Utc::now() - chrono::Duration::from_std(duration).ok()?;
        
        let recent: Vec<_> = history.iter()
            .filter(|m| m.timestamp > cutoff)
            .collect();
        
        if recent.is_empty() {
            return None;
        }
        
        let sum: f64 = recent.iter().map(|m| m.cpu.usage_percent).sum();
        Some(sum / recent.len() as f64)
    }
    
    /// Get peak memory usage
    pub fn get_peak_memory(&self, container_id: &ContainerId) -> Option<u64> {
        let history = self.get_history(container_id);
        history.iter().map(|m| m.memory.usage_bytes).max()
    }
    
    /// Get total network I/O
    pub fn get_total_network_io(&self, container_id: &ContainerId) -> Option<(u64, u64)> {
        let history = self.get_history(container_id);
        history.last().map(|m| (m.network.rx_bytes, m.network.tx_bytes))
    }
    
    /// Get network throughput (bytes per second)
    pub fn get_network_throughput(&self, container_id: &ContainerId) -> Option<(f64, f64)> {
        let history = self.get_history(container_id);
        if history.len() < 2 {
            return None;
        }
        
        let first = &history[history.len() - 2];
        let last = &history[history.len() - 1];
        
        let duration = (last.timestamp - first.timestamp).num_seconds() as f64;
        if duration <= 0.0 {
            return None;
        }
        
        let rx_diff = (last.network.rx_bytes - first.network.rx_bytes) as f64;
        let tx_diff = (last.network.tx_bytes - first.network.tx_bytes) as f64;
        
        Some((rx_diff / duration, tx_diff / duration))
    }
    
    /// Clear history for a container
    pub fn clear_history(&self, container_id: &ContainerId) {
        let mut history = self.history.lock().unwrap();
        history.remove(container_id);
    }
    
    /// Get all tracked container IDs
    pub fn get_container_ids(&self) -> Vec<ContainerId> {
        let history = self.history.lock().unwrap();
        history.keys().cloned().collect()
    }
    
    /// Generate resource usage report
    pub fn generate_report(&self) -> MetricsReport {
        let history = self.history.lock().unwrap();
        
        let mut container_reports = Vec::new();
        for (container_id, metrics) in history.iter() {
            if let Some(latest) = metrics.last() {
                let usage = ResourceUsage::from_metrics(latest);
                
                // Calculate averages
                let avg_cpu = if metrics.len() > 1 {
                    let sum: f64 = metrics.iter().map(|m| m.cpu.usage_percent).sum();
                    sum / metrics.len() as f64
                } else {
                    latest.cpu.usage_percent
                };
                
                let peak_memory = metrics.iter().map(|m| m.memory.usage_bytes).max().unwrap_or(0);
                
                container_reports.push(ContainerReport {
                    container_id: container_id.clone(),
                    current_usage: usage,
                    average_cpu_percent: avg_cpu,
                    peak_memory_bytes: peak_memory,
                    uptime_seconds: metrics.len() as u64 * 60, // Assuming 60s intervals
                });
            }
        }
        
        let total_cpu_cores: f64 = container_reports.iter().map(|r| r.current_usage.cpu_cores).sum();
        let total_memory_bytes: u64 = container_reports.iter().map(|r| r.current_usage.memory_bytes).sum();
        let total_network_rx: u64 = container_reports.iter().map(|r| r.current_usage.network_rx_bytes).sum();
        let total_network_tx: u64 = container_reports.iter().map(|r| r.current_usage.network_tx_bytes).sum();
        
        MetricsReport {
            generated_at: chrono::Utc::now(),
            container_count: container_reports.len(),
            containers: container_reports,
            total_cpu_cores,
            total_memory_bytes,
            total_network_rx,
            total_network_tx,
        }
    }
}

/// Container report entry
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContainerReport {
    pub container_id: ContainerId,
    pub current_usage: ResourceUsage,
    pub average_cpu_percent: f64,
    pub peak_memory_bytes: u64,
    pub uptime_seconds: u64,
}

/// Metrics report
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MetricsReport {
    pub generated_at: chrono::DateTime<chrono::Utc>,
    pub container_count: usize,
    pub containers: Vec<ContainerReport>,
    pub total_cpu_cores: f64,
    pub total_memory_bytes: u64,
    pub total_network_rx: u64,
    pub total_network_tx: u64,
}

impl MetricsReport {
    /// Format as JSON
    pub fn to_json(&self) -> ContainerResult<String> {
        serde_json::to_string_pretty(self)
            .map_err(|e| ContainerError::Serialization(e))
    }
    
    /// Format as human-readable text
    pub fn to_text(&self) -> String {
        let mut text = format!(
            "Container Metrics Report - {}\n",
            self.generated_at.format("%Y-%m-%d %H:%M:%S UTC")
        );
        text.push_str(&"=".repeat(60));
        text.push('\n');
        
        text.push_str(&format!("Total Containers: {}\n", self.container_count));
        text.push_str(&format!("Total CPU Cores: {:.2}\n", self.total_cpu_cores));
        text.push_str(&format!("Total Memory: {}\n", format_bytes(self.total_memory_bytes)));
        text.push_str(&format!("Total Network RX: {}\n", format_bytes(self.total_network_rx)));
        text.push_str(&format!("Total Network TX: {}\n", format_bytes(self.total_network_tx)));
        text.push('\n');
        
        for container in &self.containers {
            text.push_str(&format!("Container: {}\n", container.container_id));
            text.push_str(&format!("  CPU: {:.2}% (avg)\n", container.average_cpu_percent));
            text.push_str(&format!("  Memory: {} / {}\n",
                container.current_usage.format_memory(),
                container.current_usage.format_memory_limit()
            ));
            text.push_str(&format!("  Peak Memory: {}\n", format_bytes(container.peak_memory_bytes)));
            text.push_str(&format!("  PIDs: {}\n", container.current_usage.pids));
            text.push_str(&format!("  Uptime: {}s\n", container.uptime_seconds));
            text.push('\n');
        }
        
        text
    }
}

/// Format bytes to human-readable string
fn format_bytes(bytes: u64) -> String {
    const UNITS: &[&str] = &["B", "KB", "MB", "GB", "TB", "PB"];
    
    if bytes == 0 {
        return "0 B".to_string();
    }
    
    let exp = (bytes as f64).log(1024.0).min(UNITS.len() as f64 - 1.0) as usize;
    let value = bytes as f64 / 1024f64.powi(exp as i32);
    
    format!("{:.2} {}", value, UNITS[exp])
}

/// Metrics threshold for alerting
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MetricsThreshold {
    pub cpu_percent: Option<f64>,
    pub memory_percent: Option<f64>,
    pub pids_percent: Option<f64>,
    pub network_rx_bps: Option<u64>,
    pub network_tx_bps: Option<u64>,
}

impl MetricsThreshold {
    /// Check if metrics exceed thresholds
    pub fn check(&self, metrics: &ContainerMetrics) -> Vec<ThresholdAlert> {
        let mut alerts = Vec::new();
        
        if let Some(threshold) = self.cpu_percent {
            if metrics.cpu.usage_percent > threshold {
                alerts.push(ThresholdAlert {
                    metric: "cpu".to_string(),
                    value: metrics.cpu.usage_percent,
                    threshold,
                    severity: AlertSeverity::Warning,
                });
            }
        }
        
        if let Some(threshold) = self.memory_percent {
            let memory_percent = if metrics.memory.limit_bytes > 0 {
                (metrics.memory.usage_bytes as f64 / metrics.memory.limit_bytes as f64) * 100.0
            } else {
                0.0
            };
            
            if memory_percent > threshold {
                alerts.push(ThresholdAlert {
                    metric: "memory".to_string(),
                    value: memory_percent,
                    threshold,
                    severity: if memory_percent > threshold * 1.2 {
                        AlertSeverity::Critical
                    } else {
                        AlertSeverity::Warning
                    },
                });
            }
        }
        
        alerts
    }
}

/// Threshold alert
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ThresholdAlert {
    pub metric: String,
    pub value: f64,
    pub threshold: f64,
    pub severity: AlertSeverity,
}

/// Alert severity
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum AlertSeverity {
    Info,
    Warning,
    Critical,
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_format_bytes() {
        assert_eq!(format_bytes(0), "0 B");
        assert_eq!(format_bytes(1024), "1.00 KB");
        assert_eq!(format_bytes(1024 * 1024), "1.00 MB");
        assert_eq!(format_bytes(1024 * 1024 * 1024), "1.00 GB");
    }
    
    #[test]
    fn test_resource_usage() {
        let usage = ResourceUsage {
            cpu_cores: 1.5,
            memory_bytes: 1024 * 1024 * 1024,
            memory_limit_bytes: 2 * 1024 * 1024 * 1024,
            network_rx_bytes: 1000,
            network_tx_bytes: 2000,
            block_read_bytes: 500,
            block_write_bytes: 600,
            pids: 50,
        };
        
        assert_eq!(usage.memory_percent(), 50.0);
        assert_eq!(usage.format_memory(), "1.00 GB");
    }
    
    #[test]
    fn test_metrics_collector() {
        let collector = MetricsCollector::new();
        
        let metrics = ContainerMetrics {
            container_id: "test-1".to_string(),
            timestamp: chrono::Utc::now(),
            cpu: CpuMetrics {
                usage_percent: 50.0,
                ..Default::default()
            },
            memory: MemoryMetrics {
                usage_bytes: 1024 * 1024,
                ..Default::default()
            },
            ..Default::default()
        };
        
        collector.record(metrics.clone());
        
        let latest = collector.get_latest(&"test-1".to_string());
        assert!(latest.is_some());
        assert_eq!(latest.unwrap().cpu.usage_percent, 50.0);
    }
    
    #[test]
    fn test_threshold_check() {
        let threshold = MetricsThreshold {
            cpu_percent: Some(80.0),
            memory_percent: Some(90.0),
            pids_percent: None,
            network_rx_bps: None,
            network_tx_bps: None,
        };
        
        let metrics = ContainerMetrics {
            cpu: CpuMetrics {
                usage_percent: 85.0,
                ..Default::default()
            },
            memory: MemoryMetrics {
                usage_bytes: 950,
                limit_bytes: 1000,
                ..Default::default()
            },
            ..Default::default()
        };
        
        let alerts = threshold.check(&metrics);
        assert_eq!(alerts.len(), 2);
        assert_eq!(alerts[0].metric, "cpu");
        assert_eq!(alerts[1].metric, "memory");
    }
}
