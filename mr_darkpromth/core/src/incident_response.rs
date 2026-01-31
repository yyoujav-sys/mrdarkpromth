use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc, Duration};
use uuid::Uuid;
use std::collections::{HashMap, VecDeque};
use thiserror::Error;

#[derive(Error, Debug)]
pub enum IncidentError {
    #[error("Incident not found: {0}")]
    IncidentNotFound(Uuid),
    #[error("Invalid incident status transition: {from} -> {to}")]
    InvalidStatusTransition { from: String, to: String },
    #[error("Action not allowed: {0}")]
    ActionNotAllowed(String),
    #[error("Escalation criteria not met: {0}")]
    EscalationCriteriaNotMet(String),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum IncidentSeverity {
    Low,
    Medium,
    High,
    Critical,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum IncidentStatus {
    Open,
    Investigating,
    Contained,
    Resolved,
    Closed,
    Escalated,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum IncidentType {
    SecurityBreach,
    DDoSAttack,
    DataLeak,
    UnauthorizedAccess,
    MalwareDetection,
    SystemCompromise,
    PolicyViolation,
    SuspiciousActivity,
    Other,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum IncidentAction {
    BlockIp,
    SuspendUser,
    IsolateSystem,
    RotateCredentials,
    EnableEnhancedMonitoring,
    NotifyAdmins,
    NotifyUsers,
    BackupData,
    RestoreFromBackup,
    UpdateSecurityPolicies,
    ConductForensics,
    ExternalReporting,
    LegalAction,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Incident {
    pub id: Uuid,
    pub title: String,
    pub description: String,
    pub incident_type: IncidentType,
    pub severity: IncidentSeverity,
    pub status: IncidentStatus,
    pub source_ip: Option<String>,
    pub user_id: Option<Uuid>,
    pub affected_systems: Vec<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub resolved_at: Option<DateTime<Utc>>,
    pub assigned_to: Option<Uuid>,
    pub tags: Vec<String>,
    pub metadata: serde_json::Value,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IncidentActionLog {
    pub id: Uuid,
    pub incident_id: Uuid,
    pub action: IncidentAction,
    pub description: String,
    pub performed_by: Option<Uuid>,
    pub performed_at: DateTime<Utc>,
    pub success: bool,
    pub result: Option<serde_json::Value>,
    pub error_message: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EscalationRule {
    pub id: Uuid,
    pub name: String,
    pub incident_type: Option<IncidentType>,
    pub severity_threshold: Option<IncidentSeverity>,
    pub time_threshold: Option<Duration>,
    pub action_count_threshold: Option<u32>,
    pub escalation_action: IncidentAction,
    pub enabled: bool,
}

#[derive(Debug, Clone)]
pub struct IncidentResponse {
    incidents: HashMap<Uuid, Incident>,
    action_logs: HashMap<Uuid, Vec<IncidentActionLog>>,
    escalation_rules: Vec<EscalationRule>,
    auto_response_enabled: bool,
    notification_channels: Vec<NotificationChannel>,
    response_templates: HashMap<String, ResponseTemplate>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NotificationChannel {
    pub id: Uuid,
    pub name: String,
    pub channel_type: NotificationType,
    pub config: serde_json::Value,
    pub enabled: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum NotificationType {
    Email,
    Slack,
    Webhook,
    SMS,
    SystemLog,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResponseTemplate {
    pub id: Uuid,
    pub name: String,
    pub incident_type: IncidentType,
    pub severity: IncidentSeverity,
    pub actions: Vec<IncidentAction>,
    pub conditions: Vec<String>,
    pub enabled: bool,
}

impl IncidentResponse {
    pub fn new() -> Self {
        Self {
            incidents: HashMap::new(),
            action_logs: HashMap::new(),
            escalation_rules: Vec::new(),
            auto_response_enabled: true,
            notification_channels: Vec::new(),
            response_templates: HashMap::new(),
        }
    }

    pub fn create_incident(&mut self, 
                          title: String,
                          description: String,
                          incident_type: IncidentType,
                          severity: IncidentSeverity,
                          source_ip: Option<String>,
                          user_id: Option<Uuid>,
                          affected_systems: Vec<String>) -> Uuid {
        let incident = Incident {
            id: Uuid::new_v4(),
            title,
            description,
            incident_type,
            severity: severity.clone(),
            status: IncidentStatus::Open,
            source_ip,
            user_id,
            affected_systems,
            created_at: Utc::now(),
            updated_at: Utc::now(),
            resolved_at: None,
            assigned_to: None,
            tags: Vec::new(),
            metadata: serde_json::json!({}),
        };

        let incident_id = incident.id;
        self.incidents.insert(incident_id, incident.clone());

        // Check for auto-response
        if self.auto_response_enabled {
            self.check_auto_response(&incident_id);
        }

        // Send notifications
        self.send_incident_notifications(&incident);

        incident_id
    }

    pub fn update_incident_status(&mut self, 
                                 incident_id: Uuid, 
                                 new_status: IncidentStatus,
                                 performed_by: Option<Uuid>) -> Result<(), IncidentError> {
        let incident = self.incidents.get_mut(&incident_id)
            .ok_or_else(|| IncidentError::IncidentNotFound(incident_id))?;

        // Validate status transition
        if !self.is_valid_status_transition(&incident.status, &new_status) {
            return Err(IncidentError::InvalidStatusTransition {
                from: format!("{:?}", incident.status),
                to: format!("{:?}", new_status),
            });
        }

        incident.status = new_status.clone();
        incident.updated_at = Utc::now();

        if matches!(new_status, IncidentStatus::Resolved | IncidentStatus::Closed) {
            incident.resolved_at = Some(Utc::now());
        }

        // Log the action
        self.log_action(incident_id, 
                       IncidentAction::UpdateStatus, 
                       format!("Status changed to {:?}", new_status),
                       performed_by,
                       true,
                       None,
                       None);

        // Check for escalation
        self.check_escalation(&incident_id);

        Ok(())
    }

    pub fn execute_action(&mut self, 
                         incident_id: Uuid,
                         action: IncidentAction,
                         description: String,
                         performed_by: Option<Uuid>) -> Result<(), IncidentError> {
        let incident = self.incidents.get(&incident_id)
            .ok_or_else(|| IncidentError::IncidentNotFound(incident_id))?;

        // Check if action is allowed for current status
        if !self.is_action_allowed(&incident.status, &action) {
            return Err(IncidentError::ActionNotAllowed(
                format!("Action {:?} not allowed in status {:?}", action, incident.status)
            ));
        }

        // Execute the action
        let (success, result, error) = self.perform_action(&action, &incident);

        // Log the action
        self.log_action(incident_id, action.clone(), description, performed_by, success, result, error);

        // Update incident if action was successful
        if success {
            if let Some(incident) = self.incidents.get_mut(&incident_id) {
                incident.updated_at = Utc::now();
            }
        }

        Ok(())
    }

    pub fn assign_incident(&mut self, incident_id: Uuid, assigned_to: Uuid) -> Result<(), IncidentError> {
        let incident = self.incidents.get_mut(&incident_id)
            .ok_or_else(|| IncidentError::IncidentNotFound(incident_id))?;

        incident.assigned_to = Some(assigned_to);
        incident.updated_at = Utc::now();

        Ok(())
    }

    pub fn get_incident(&self, incident_id: Uuid) -> Option<&Incident> {
        self.incidents.get(&incident_id)
    }

    pub fn get_incidents_by_status(&self, status: IncidentStatus) -> Vec<&Incident> {
        self.incidents.values()
            .filter(|incident| incident.status == status)
            .collect()
    }

    pub fn get_incidents_by_severity(&self, severity: IncidentSeverity) -> Vec<&Incident> {
        self.incidents.values()
            .filter(|incident| incident.severity == severity)
            .collect()
    }

    pub fn get_incidents_by_type(&self, incident_type: IncidentType) -> Vec<&Incident> {
        self.incidents.values()
            .filter(|incident| incident.incident_type == incident_type)
            .collect()
    }

    pub fn get_action_logs(&self, incident_id: Uuid) -> Option<&Vec<IncidentActionLog>> {
        self.action_logs.get(&incident_id)
    }

    pub fn add_escalation_rule(&mut self, rule: EscalationRule) {
        self.escalation_rules.push(rule);
    }

    pub fn add_notification_channel(&mut self, channel: NotificationChannel) {
        self.notification_channels.push(channel);
    }

    pub fn add_response_template(&mut self, template: ResponseTemplate) {
        self.response_templates.insert(template.name.clone(), template);
    }

    pub fn get_incident_stats(&self) -> IncidentStats {
        let total_incidents = self.incidents.len();
        let open_incidents = self.incidents.values()
            .filter(|i| matches!(i.status, IncidentStatus::Open | IncidentStatus::Investigating))
            .count();
        let resolved_incidents = self.incidents.values()
            .filter(|i| matches!(i.status, IncidentStatus::Resolved | IncidentStatus::Closed))
            .count();

        let incidents_by_type = self.incidents.values()
            .fold(HashMap::new(), |mut acc, incident| {
                *acc.entry(incident.incident_type.clone()).or_insert(0) += 1;
                acc
            });

        let incidents_by_severity = self.incidents.values()
            .fold(HashMap::new(), |mut acc, incident| {
                *acc.entry(incident.severity.clone()).or_insert(0) += 1;
                acc
            });

        let avg_resolution_time = self.calculate_average_resolution_time();

        IncidentStats {
            total_incidents,
            open_incidents,
            resolved_incidents,
            incidents_by_type,
            incidents_by_severity,
            average_resolution_time_hours: avg_resolution_time,
            auto_response_enabled: self.auto_response_enabled,
        }
    }

    // Private helper methods
    fn is_valid_status_transition(&self, from: &IncidentStatus, to: &IncidentStatus) -> bool {
        use IncidentStatus::*;
        
        match (from, to) {
            (Open, Investigating) => true,
            (Open, Contained) => true,
            (Open, Resolved) => true,
            (Open, Escalated) => true,
            (Investigating, Contained) => true,
            (Investigating, Escalated) => true,
            (Investigating, Resolved) => true,
            (Contained, Resolved) => true,
            (Resolved, Closed) => true,
            (Escalated, Investigating) => true,
            (Escalated, Contained) => true,
            (Escalated, Resolved) => true,
            _ => false,
        }
    }

    fn is_action_allowed(&self, status: &IncidentStatus, action: &IncidentAction) -> bool {
        use IncidentStatus::*;
        use IncidentAction::*;
        
        match status {
            Open => matches!(action, 
                BlockIp | SuspendUser | NotifyAdmins | EnableEnhancedMonitoring | UpdateSecurityPolicies),
            Investigating => matches!(action,
                BlockIp | SuspendUser | IsolateSystem | ConductForensics | NotifyAdmins | EnableEnhancedMonitoring),
            Contained => matches!(action,
                RotateCredentials | BackupData | RestoreFromBackup | UpdateSecurityPolicies | NotifyUsers),
            Resolved => matches!(action,
                UpdateSecurityPolicies | ExternalReporting | LegalAction),
            Closed => false,
            Escalated => true, // All actions allowed when escalated
        }
    }

    fn perform_action(&self, action: &IncidentAction, incident: &Incident) -> (bool, Option<serde_json::Value>, Option<String>) {
        // Simulate action execution
        match action {
            IncidentAction::BlockIp => {
                if let Some(ip) = &incident.source_ip {
                    log::info!("Blocking IP: {}", ip);
                    (true, Some(serde_json::json!({"blocked_ip": ip})), None)
                } else {
                    (false, None, Some("No IP address to block".to_string()))
                }
            },
            IncidentAction::SuspendUser => {
                if let Some(user_id) = &incident.user_id {
                    log::info!("Suspending user: {}", user_id);
                    (true, Some(serde_json::json!({"suspended_user": user_id})), None)
                } else {
                    (false, None, Some("No user to suspend".to_string()))
                }
            },
            IncidentAction::NotifyAdmins => {
                log::info!("Notifying admins about incident: {}", incident.id);
                (true, Some(serde_json::json!({"notification_sent": true})), None)
            },
            _ => {
                log::info!("Executing action: {:?}", action);
                (true, Some(serde_json::json!({"action_executed": format!("{:?}", action)})), None)
            }
        }
    }

    fn log_action(&mut self, 
                  incident_id: Uuid,
                  action: IncidentAction,
                  description: String,
                  performed_by: Option<Uuid>,
                  success: bool,
                  result: Option<serde_json::Value>,
                  error_message: Option<String>) {
        let action_log = IncidentActionLog {
            id: Uuid::new_v4(),
            incident_id,
            action,
            description,
            performed_by,
            performed_at: Utc::now(),
            success,
            result,
            error_message,
        };

        self.action_logs.entry(incident_id).or_insert_with(Vec::new).push(action_log);
    }

    fn check_auto_response(&mut self, incident_id: &Uuid) {
        let incident = match self.incidents.get(incident_id) {
            Some(i) => i,
            None => return,
        };

        // Find matching response template
        for template in self.response_templates.values() {
            if template.incident_type == incident.incident_type 
                && template.severity == incident.severity 
                && template.enabled {
                
                for action in &template.actions {
                    if self.is_action_allowed(&incident.status, action) {
                        let _ = self.execute_action(*incident_id, action.clone(), 
                                                   format!("Auto-response: {:?}", action), None);
                    }
                }
                break;
            }
        }
    }

    fn check_escalation(&mut self, incident_id: &Uuid) {
        let incident = match self.incidents.get(incident_id) {
            Some(i) => i,
            None => return,
        };

        for rule in &self.escalation_rules {
            if !rule.enabled {
                continue;
            }

            let should_escalate = self.evaluate_escalation_rule(rule, incident);
            
            if should_escalate {
                let _ = self.execute_action(*incident_id, rule.escalation_action.clone(),
                                           format!("Escalation: {}", rule.name), None);
                let _ = self.update_incident_status(*incident_id, IncidentStatus::Escalated, None);
                break;
            }
        }
    }

    fn evaluate_escalation_rule(&self, rule: &EscalationRule, incident: &Incident) -> bool {
        // Check incident type
        if let Some(rule_type) = &rule.incident_type {
            if rule_type != &incident.incident_type {
                return false;
            }
        }

        // Check severity threshold
        if let Some(rule_severity) = &rule.severity_threshold {
            if !self.meets_severity_threshold(&incident.severity, rule_severity) {
                return false;
            }
        }

        // Check time threshold
        if let Some(time_threshold) = rule.time_threshold {
            if Utc::now() - incident.created_at < time_threshold {
                return false;
            }
        }

        // Check action count threshold
        if let Some(action_threshold) = rule.action_count_threshold {
            let action_count = self.action_logs.get(&incident.id)
                .map(|logs| logs.len() as u32)
                .unwrap_or(0);
            
            if action_count < action_threshold {
                return false;
            }
        }

        true
    }

    fn meets_severity_threshold(&self, current: &IncidentSeverity, threshold: &IncidentSeverity) -> bool {
        use IncidentSeverity::*;
        
        match (current, threshold) {
            (Low, Low) => true,
            (Low, Medium) | (Low, High) | (Low, Critical) => false,
            (Medium, Low) => true,
            (Medium, Medium) => true,
            (Medium, High) | (Medium, Critical) => false,
            (High, Low) | (High, Medium) | (High, High) => true,
            (High, Critical) => false,
            (Critical, _) => true,
        }
    }

    fn send_incident_notifications(&self, incident: &Incident) {
        for channel in &self.notification_channels {
            if channel.enabled {
                self.send_notification(channel, incident);
            }
        }
    }

    fn send_notification(&self, channel: &NotificationChannel, incident: &Incident) {
        log::info!("Sending notification via {}: Incident {} - {}", 
                  channel.name, incident.id, incident.title);
        // Implementation would depend on channel type
    }

    fn calculate_average_resolution_time(&self) -> Option<f64> {
        let resolved_incidents: Vec<_> = self.incidents.values()
            .filter(|i| i.resolved_at.is_some())
            .collect();

        if resolved_incidents.is_empty() {
            return None;
        }

        let total_duration: Duration = resolved_incidents.iter()
            .map(|i| i.resolved_at.unwrap() - i.created_at)
            .sum();

        Some(total_duration.num_milliseconds() as f64 / (resolved_incidents.len() as f64 * 3600000.0))
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IncidentStats {
    pub total_incidents: usize,
    pub open_incidents: usize,
    pub resolved_incidents: usize,
    pub incidents_by_type: HashMap<IncidentType, u64>,
    pub incidents_by_severity: HashMap<IncidentSeverity, u64>,
    pub average_resolution_time_hours: Option<f64>,
    pub auto_response_enabled: bool,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_incident_creation() {
        let mut ir = IncidentResponse::new();
        
        let incident_id = ir.create_incident(
            "Test Incident".to_string(),
            "Test description".to_string(),
            IncidentType::SecurityBreach,
            IncidentSeverity::High,
            Some("192.168.1.1".to_string()),
            None,
            vec!["system1".to_string()],
        );

        let incident = ir.get_incident(incident_id).unwrap();
        assert_eq!(incident.title, "Test Incident");
        assert_eq!(incident.severity, IncidentSeverity::High);
        assert!(matches!(incident.status, IncidentStatus::Open));
    }

    #[test]
    fn test_status_transition() {
        let mut ir = IncidentResponse::new();
        
        let incident_id = ir.create_incident(
            "Test".to_string(),
            "Test".to_string(),
            IncidentType::SecurityBreach,
            IncidentSeverity::Medium,
            None,
            None,
            vec![],
        );

        // Valid transition
        assert!(ir.update_incident_status(incident_id, IncidentStatus::Investigating, None).is_ok());

        // Invalid transition
        assert!(ir.update_incident_status(incident_id, IncidentStatus::Open, None).is_err());
    }

    #[test]
    fn test_action_execution() {
        let mut ir = IncidentResponse::new();
        
        let incident_id = ir.create_incident(
            "Test".to_string(),
            "Test".to_string(),
            IncidentType::SecurityBreach,
            IncidentSeverity::High,
            Some("192.168.1.1".to_string()),
            None,
            vec![],
        );

        assert!(ir.execute_action(incident_id, IncidentAction::BlockIp, 
                                 "Block malicious IP".to_string(), None).is_ok());

        let logs = ir.get_action_logs(incident_id).unwrap();
        assert_eq!(logs.len(), 1);
        assert!(logs[0].success);
    }
}
