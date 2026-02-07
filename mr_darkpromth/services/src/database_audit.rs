use std::str::FromStr;
use std::net::IpAddr;
use async_trait::async_trait;
use anyhow::Result;
use chrono::{DateTime, Utc};
use mr_darkpromth_core::{
    audit::{AuditLog, AuditFilter, AuditStats, AuditAction, AuditSeverity},
    tier::UserTier,
};
use mr_darkpromth_db::DbPool;
use sqlx::Row;

pub struct DatabaseAuditService {
    pool: DbPool,
}

impl DatabaseAuditService {
    pub fn new(pool: DbPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl crate::audit::AuditService for DatabaseAuditService {
    async fn log_audit(&self, audit_log: AuditLog) -> Result<()> {
        let ip_address: Option<String> = audit_log
            .ip_address
            .and_then(|value| value.parse::<IpAddr>().ok().map(|_| value));
        let query = r#"
            INSERT INTO audit_logs (
                id, user_id, action, severity, user_tier, ip_address, user_agent,
                request_id, details, timestamp, success, error_message
            ) VALUES ($1, $2, $3, $4, $5, $6::inet, $7, $8, $9, $10, $11, $12)
        "#;

        sqlx::query(query)
            .bind(audit_log.id)
            .bind(audit_log.user_id)
            .bind(audit_log.action.to_string())
            .bind(audit_log.severity.to_string())
            .bind(audit_log.user_tier.map(|t| t.as_str().to_string()))
            .bind(ip_address)
            .bind(audit_log.user_agent)
            .bind(audit_log.request_id)
            .bind(serde_json::to_value(&audit_log.details)?)
            .bind(audit_log.timestamp)
            .bind(audit_log.success)
            .bind(audit_log.error_message)
            .execute(&self.pool)
            .await?;

        Ok(())
    }

    async fn get_audit_logs(&self, filter: AuditFilter) -> Result<Vec<AuditLog>> {
        let mut query = "SELECT * FROM audit_logs WHERE 1=1".to_string();
        let mut bind_count = 0;

        if filter.user_id.is_some() {
            bind_count += 1;
            query.push_str(&format!(" AND user_id = ${}", bind_count));
        }

        if filter.action.is_some() {
            bind_count += 1;
            query.push_str(&format!(" AND action = ${}", bind_count));
        }

        if filter.severity.is_some() {
            bind_count += 1;
            query.push_str(&format!(" AND severity = ${}", bind_count));
        }

        if filter.user_tier.is_some() {
            bind_count += 1;
            query.push_str(&format!(" AND user_tier = ${}", bind_count));
        }

        if filter.start_time.is_some() {
            bind_count += 1;
            query.push_str(&format!(" AND timestamp >= ${}", bind_count));
        }

        if filter.end_time.is_some() {
            bind_count += 1;
            query.push_str(&format!(" AND timestamp <= ${}", bind_count));
        }

        if filter.success_only.is_some() {
            bind_count += 1;
            query.push_str(&format!(" AND success = ${}", bind_count));
        }

        query.push_str(" ORDER BY timestamp DESC");

        if filter.limit.is_some() {
            bind_count += 1;
            query.push_str(&format!(" LIMIT ${}", bind_count));
        }

        if filter.offset.is_some() {
            bind_count += 1;
            query.push_str(&format!(" OFFSET ${}", bind_count));
        }

        let mut query_builder = sqlx::query(&query);

        if let Some(user_id) = filter.user_id {
            query_builder = query_builder.bind(user_id);
        }

        if let Some(action) = filter.action {
            query_builder = query_builder.bind(action.to_string());
        }

        if let Some(severity) = filter.severity {
            query_builder = query_builder.bind(severity.to_string());
        }

        if let Some(user_tier) = filter.user_tier {
            query_builder = query_builder.bind(user_tier.as_str());
        }

        if let Some(start_time) = filter.start_time {
            query_builder = query_builder.bind(start_time);
        }

        if let Some(end_time) = filter.end_time {
            query_builder = query_builder.bind(end_time);
        }

        if let Some(success_only) = filter.success_only {
            query_builder = query_builder.bind(success_only);
        }

        if let Some(limit) = filter.limit {
            query_builder = query_builder.bind(limit as i64);
        }

        if let Some(offset) = filter.offset {
            query_builder = query_builder.bind(offset as i64);
        }

        let rows = query_builder.fetch_all(&self.pool).await?;

        let mut logs = Vec::new();
        for row in rows {
            let action_str: String = row.get("action");
            let severity_str: String = row.get("severity");
            let user_tier_str: Option<String> = row.get("user_tier");

            logs.push(AuditLog {
                id: row.get("id"),
                user_id: row.get("user_id"),
                action: AuditAction::from_str(&action_str).map_err(|e| anyhow::anyhow!(e))?,
                severity: AuditSeverity::from_str(&severity_str).map_err(|e| anyhow::anyhow!(e))?,
                user_tier: user_tier_str.map(|s| UserTier::from_str(&s).map_err(|e| anyhow::anyhow!(e))).transpose()?,
                ip_address: row.get("ip_address"),
                user_agent: row.get("user_agent"),
                request_id: row.get("request_id"),
                details: row.get("details"),
                timestamp: row.get("timestamp"),
                success: row.get("success"),
                error_message: row.get("error_message"),
            });
        }

        Ok(logs)
    }

    async fn get_audit_stats(&self, filter: AuditFilter) -> Result<AuditStats> {
        // This is a simplified implementation - in production you'd want more sophisticated queries
        let logs = self.get_audit_logs(filter).await?;

        let total_logs = logs.len() as u64;
        let successful_operations = logs.iter().filter(|l| l.success).count() as u64;
        let failed_operations = total_logs - successful_operations;
        let jailbreak_attempts = logs.iter()
            .filter(|l| matches!(l.action, AuditAction::JailbreakAttempt))
            .count() as u64;
        let ultra_tier_operations = logs.iter()
            .filter(|l| matches!(l.user_tier, Some(UserTier::Ultra)))
            .count() as u64;

        let processing_times: Vec<f64> = logs.iter()
            .filter_map(|l| {
                l.details.get("processing_time_ms")
                    .and_then(|v| v.as_u64())
                    .map(|ms| ms as f64)
            })
            .collect();

        let average_processing_time = if processing_times.is_empty() {
            0.0
        } else {
            processing_times.iter().sum::<f64>() / processing_times.len() as f64
        };

        let mut user_counts = std::collections::HashMap::new();
        let mut action_counts = std::collections::HashMap::new();

        for log in &logs {
            if let Some(user_id) = log.user_id {
                *user_counts.entry(user_id).or_insert(0) += 1;
            }
            *action_counts.entry(log.action.clone()).or_insert(0) += 1;
        }

        let most_active_users = user_counts.into_iter()
            .map(|(id, count)| (id, count))
            .take(10)
            .collect();

        let most_common_actions = action_counts.into_iter()
            .map(|(action, count)| (action, count))
            .take(10)
            .collect();

        Ok(AuditStats {
            total_logs,
            successful_operations,
            failed_operations,
            jailbreak_attempts,
            ultra_tier_operations,
            average_processing_time,
            most_active_users,
            most_common_actions,
        })
    }

    async fn cleanup_old_logs(&self, older_than: DateTime<Utc>) -> Result<u64> {
        let result = sqlx::query("DELETE FROM audit_logs WHERE timestamp < $1")
            .bind(older_than)
            .execute(&self.pool)
            .await?;

        Ok(result.rows_affected())
    }
}
