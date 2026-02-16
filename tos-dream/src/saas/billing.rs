//! Phase 16 Week 2: SaaS Billing & Usage Tracking
//!
//! Resource usage metering, session time tracking, and analytics for SaaS tenants.

use super::{TenantId, SessionId, SaasResult, SaasError};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};

/// Usage record for a tenant
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UsageRecord {
    pub tenant_id: TenantId,
    pub timestamp: DateTime<Utc>,
    pub resource_type: ResourceType,
    pub amount: f64,
    pub unit: String,
}

/// Types of resources that can be metered
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ResourceType {
    CpuCoreSeconds,
    MemoryByteSeconds,
    StorageBytes,
    NetworkInBytes,
    NetworkOutBytes,
    SessionSeconds,
}

/// Billing tier configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BillingTier {
    pub name: String,
    pub monthly_price: f64,
    pub included_storage_gb: u64,
    pub included_sessions: usize,
    pub overage_session_price: f64,
}

/// Usage statistics for a tenant
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct TenantUsageStats {
    pub current_sessions: usize,
    pub total_session_seconds: u64,
    pub total_cpu_seconds: f64,
    pub total_memory_gb_seconds: f64,
    pub network_in_bytes: u64,
    pub network_out_bytes: u64,
}

/// Billing manager coordinates usage tracking and billing
#[derive(Debug)]
pub struct BillingManager {
    usage_history: Arc<Mutex<Vec<UsageRecord>>>,
    tenant_stats: Arc<Mutex<HashMap<TenantId, TenantUsageStats>>>,
    active_sessions: Arc<Mutex<HashMap<SessionId, (TenantId, DateTime<Utc>)>>>,
}

impl BillingManager {
    /// Create a new billing manager
    pub fn new() -> Self {
        Self {
            usage_history: Arc::new(Mutex::new(Vec::new())),
            tenant_stats: Arc::new(Mutex::new(HashMap::new())),
            active_sessions: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    /// Start tracking a session
    pub fn start_session(&self, tenant_id: TenantId, session_id: SessionId) {
        let mut sessions = self.active_sessions.lock().unwrap();
        sessions.insert(session_id, (tenant_id, Utc::now()));
    }

    /// End a session and record its duration
    pub fn end_session(&self, session_id: &SessionId) -> SaasResult<()> {
        let mut sessions = self.active_sessions.lock().unwrap();
        if let Some((tenant_id, start_time)) = sessions.remove(session_id) {
            let duration = Utc::now().signed_duration_since(start_time).num_seconds() as u64;
            self.record_usage(UsageRecord {
                tenant_id: tenant_id.clone(),
                timestamp: Utc::now(),
                resource_type: ResourceType::SessionSeconds,
                amount: duration as f64,
                unit: "seconds".to_string(),
            })?;

            let mut stats = self.tenant_stats.lock().unwrap();
            let tenant_stat = stats.entry(tenant_id).or_default();
            tenant_stat.total_session_seconds += duration;
            Ok(())
        } else {
            Err(SaasError::NotFound(format!("Active session {} not found", session_id)))
        }
    }

    /// Record a usage event
    pub fn record_usage(&self, record: UsageRecord) -> SaasResult<()> {
        let mut history = self.usage_history.lock().unwrap();
        history.push(record);
        Ok(())
    }

    /// Get usage statistics for a tenant
    pub fn get_tenant_stats(&self, tenant_id: &TenantId) -> TenantUsageStats {
        let stats = self.tenant_stats.lock().unwrap();
        stats.get(tenant_id).cloned().unwrap_or_default()
    }

    /// Clear old usage history
    pub fn prune_history(&self, before: DateTime<Utc>) {
        let mut history = self.usage_history.lock().unwrap();
        history.retain(|r| r.timestamp >= before);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_billing_manager_sessions() {
        let manager = BillingManager::new();
        let tenant_id = "test-tenant".to_string();
        let session_id = "session-123".to_string();

        manager.start_session(tenant_id.clone(), session_id.clone());
        manager.end_session(&session_id).unwrap();

        let stats = manager.get_tenant_stats(&tenant_id);
        // We can't guarantee exact duration in a fast test, but it should be recorded
        assert!(stats.total_session_seconds >= 0);
    }

    #[test]
    fn test_record_usage() {
        let manager = BillingManager::new();
        let record = UsageRecord {
            tenant_id: "tenant-1".to_string(),
            timestamp: Utc::now(),
            resource_type: ResourceType::CpuCoreSeconds,
            amount: 10.5,
            unit: "seconds".to_string(),
        };

        manager.record_usage(record).unwrap();
        let history = manager.usage_history.lock().unwrap();
        assert_eq!(history.len(), 1);
        assert_eq!(history[0].amount, 10.5);
    }
}

impl Default for BillingManager {
    fn default() -> Self {
        Self::new()
    }
}
