use crate::agent_framework::{Agent, AgentState, AgentError, AgentResult};
use crate::coordinator_agent::Task;
use std::sync::Arc;
use reqwest::Client;

pub struct DecisionEngine {
    http_client: Client,
}

impl Default for DecisionEngine {
    fn default() -> Self {
        Self::new()
    }
}

impl DecisionEngine {
    pub fn new() -> Self {
        Self {
            http_client: Client::new(),
        }
    }
    
    async fn call_cerebras(&self, prompt: &str) -> AgentResult<String> {
        let api_key = std::env::var("CEREBRAS_API_KEY").unwrap_or_else(|_| "demo_key".to_string());
        
        let response = self.http_client
            .post("https://api.cerebras.ai/v1/chat/completions")
            .header("Authorization", format!("Bearer {}", api_key))
            .json(&serde_json::json!({
                "model": "llama3.1-70b",
                "messages": [
                    {"role": "system", "content": "You are a helpful assistant. Respond with valid JSON."},
                    {"role": "user", "content": prompt}
                ],
                "temperature": 0.7,
                "max_tokens": 1000
            }))
            .send()
            .await
            .map_err(|e| AgentError::CommunicationError(format!("API request failed: {}", e)))?;
        
        if !response.status().is_success() {
            return Err(AgentError::CommunicationError(format!("API returned error: {}", response.status())));
        }
        
        let response_json: serde_json::Value = response
            .json()
            .await
            .map_err(|e| AgentError::CommunicationError(format!("Failed to parse response: {}", e)))?;
        
        let content = response_json
            .get("choices")
            .and_then(|c| c.get(0))
            .and_then(|c| c.get("message"))
            .and_then(|m| m.get("content"))
            .and_then(|c| c.as_str())
            .ok_or_else(|| AgentError::CommunicationError("No content in response".to_string()))?
            .to_string();
        
        Ok(content)
    }
    
    pub async fn evaluate_task_complexity(&self, task: &Task) -> AgentResult<Complexity> {
        let prompt = format!(
            "Evaluate the complexity of this task and categorize it:\n\
            Task: {}\n\n\
            Rate complexity (1-10) and determine if it requires:\n\
            - Simple: Single agent can handle\n\
            - Moderate: Requires 2 agents\n\
            - Complex: Requires all 3 agents\n\n\
            Respond with JSON: {{\"complexity_score\": number, \"category\": \"simple|moderate|complex\", \"reasoning\": \"string\"}}",
            task.description
        );
        
        let response = self.call_cerebras(&prompt).await
            .map_err(|e| AgentError::MessageProcessingFailed(e.to_string()))?;
        
        let evaluation: serde_json::Value = serde_json::from_str(&response)
            .map_err(|e| AgentError::SerializationError(e.to_string()))?;
        
        let category = evaluation["category"].as_str().unwrap_or("moderate");
        
        let complexity = match category {
            "simple" => Complexity::Simple,
            "moderate" => Complexity::Moderate,
            "complex" => Complexity::Complex,
            _ => Complexity::Moderate,
        };
        
        Ok(complexity)
    }
    
    pub async fn determine_agent_assignment(&self, _task: &Task, complexity: Complexity) -> AgentResult<Vec<String>> {
        let agents_needed = match complexity {
            Complexity::Simple => vec!["editor".to_string()],
            Complexity::Moderate => vec!["editor".to_string(), "terminal".to_string()],
            Complexity::Complex => vec!["editor".to_string(), "terminal".to_string()],
        };
        
        Ok(agents_needed)
    }
    
    pub async fn make_decision(&self, context: &DecisionContext) -> AgentResult<Decision> {
        let prompt = format!(
            "Make a decision based on this context:\n\
            Current Task: {}\n\
            Agent State: {:?}\n\
            Available Resources: {:?}\n\
            Constraints: {:?}\n\n\
            Determine the best course of action and respond with JSON:\n\
            {{\"action\": \"proceed|delegate|retry|abort\", \"reasoning\": \"string\", \"next_steps\": [\"step1\", \"step2\"]}}",
            context.task_description,
            context.current_state,
            context.available_resources,
            context.constraints
        );
        
        let response = self.call_cerebras(&prompt).await
            .map_err(|e| AgentError::MessageProcessingFailed(e.to_string()))?;
        
        let decision_json: serde_json::Value = serde_json::from_str(&response)
            .map_err(|e| AgentError::SerializationError(e.to_string()))?;
        
        let action = decision_json["action"].as_str().unwrap_or("proceed");
        let reasoning = decision_json["reasoning"].as_str().unwrap_or("");
        let next_steps: Vec<String> = decision_json["next_steps"]
            .as_array()
            .map(|arr| arr.iter().filter_map(|v| v.as_str().map(|s| s.to_string())).collect())
            .unwrap_or_default();
        
        let decision = match action {
            "proceed" => Decision::Proceed { reasoning: reasoning.to_string(), next_steps },
            "delegate" => Decision::Delegate { 
                reasoning: reasoning.to_string(), 
                target_agent: "editor".to_string(),
                next_steps 
            },
            "retry" => Decision::Retry { reasoning: reasoning.to_string(), max_attempts: 3 },
            "abort" => Decision::Abort { reasoning: reasoning.to_string() },
            _ => Decision::Proceed { reasoning: reasoning.to_string(), next_steps },
        };
        
        Ok(decision)
    }
    
    pub async fn optimize_workflow(&self, tasks: &[Task]) -> AgentResult<Vec<Task>> {
        let prompt = format!(
            "Optimize the execution order of these tasks:\n\
            {}\n\n\
            Consider dependencies, priorities, and efficiency.\n\
            Respond with ordered task IDs as JSON array.",
            tasks.iter().map(|t| format!("{}: {}", t.id, t.description)).collect::<Vec<_>>().join("\n")
        );
        
        let response = self.call_cerebras(&prompt).await
            .map_err(|e| AgentError::MessageProcessingFailed(e.to_string()))?;
        
        let ordered_ids: Vec<String> = serde_json::from_str(&response)
            .unwrap_or_else(|_| tasks.iter().map(|t| t.id.to_string()).collect());
        
        let mut task_map: std::collections::HashMap<String, Task> = tasks.iter()
            .map(|t| (t.id.clone(), t.clone()))
            .collect();
        
        let mut optimized = Vec::new();
        for id_str in ordered_ids {
            if let Some(task) = task_map.remove(&id_str) {
                optimized.push(task);
            }
        }
        
        Ok(optimized)
    }
}

#[derive(Debug, Clone)]
pub enum Complexity {
    Simple,
    Moderate,
    Complex,
}

#[derive(Debug, Clone)]
pub struct DecisionContext {
    pub task_description: String,
    pub current_state: AgentState,
    pub available_resources: Vec<String>,
    pub constraints: Vec<String>,
}

#[derive(Debug, Clone)]
pub enum Decision {
    Proceed { reasoning: String, next_steps: Vec<String> },
    Delegate { reasoning: String, target_agent: String, next_steps: Vec<String> },
    Retry { reasoning: String, max_attempts: u32 },
    Abort { reasoning: String },
}

pub struct CollaborationPattern {
    #[allow(dead_code)]
    pattern_type: PatternType,
    #[allow(dead_code)]
    description: String,
}

#[derive(Debug, Clone)]
pub enum PatternType {
    Sequential,
    Parallel,
    Hierarchical,
    PeerToPeer,
}

impl CollaborationPattern {
    pub fn new(pattern_type: PatternType, description: &str) -> Self {
        Self {
            pattern_type,
            description: description.to_string(),
        }
    }
    
    pub fn execute_sequential(&self, agents: Vec<Arc<dyn Agent>>, tasks: Vec<String>) -> AgentResult<()> {
        log::info!("Executing sequential collaboration pattern");
        
        for (agent, task) in agents.iter().zip(tasks.iter()) {
            log::info!("Agent {} executing: {}", agent.agent_id(), task);
        }
        
        Ok(())
    }
    
    pub fn execute_parallel(&self, agents: Vec<Arc<dyn Agent>>, tasks: Vec<String>) -> AgentResult<()> {
        log::info!("Executing parallel collaboration pattern");
        
        for (agent, task) in agents.iter().zip(tasks.iter()) {
            log::info!("Agent {} executing: {}", agent.agent_id(), task);
        }
        
        Ok(())
    }
    
    pub fn execute_hierarchical(&self, coordinator: Arc<dyn Agent>, workers: Vec<Arc<dyn Agent>>, tasks: Vec<String>) -> AgentResult<()> {
        log::info!("Executing hierarchical collaboration pattern");
        log::info!("Coordinator {} orchestrating {} workers", coordinator.agent_id(), workers.len());
        
        for (worker, task) in workers.iter().zip(tasks.iter()) {
            log::info!("Worker {} assigned: {}", worker.agent_id(), task);
        }
        
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_complexity_classification() {
        let simple = Complexity::Simple;
        let moderate = Complexity::Moderate;
        let complex = Complexity::Complex;
        
        assert!(matches!(simple, Complexity::Simple));
        assert!(matches!(moderate, Complexity::Moderate));
        assert!(matches!(complex, Complexity::Complex));
    }
}
