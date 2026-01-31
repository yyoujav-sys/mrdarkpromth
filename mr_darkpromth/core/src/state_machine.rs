use crate::agent::{Agent, AgentState, AgentMessage, Task, TaskStatus};
use anyhow::Result;
use std::collections::HashMap;
use std::time::Duration;
use tokio::time::sleep;

pub struct StateMachine {
    pub current_state: AgentState,
    state_history: Vec<(AgentState, chrono::DateTime<chrono::Utc>)>,
    transition_rules: HashMap<(AgentState, AgentState), bool>,
}

impl StateMachine {
    pub fn new(initial_state: AgentState) -> Self {
        let mut transition_rules = HashMap::new();
        
        // Define valid state transitions
        transition_rules.insert((AgentState::Idle, AgentState::Thinking), true);
        transition_rules.insert((AgentState::Thinking, AgentState::Acting), true);
        transition_rules.insert((AgentState::Thinking, AgentState::Idle), true);
        transition_rules.insert((AgentState::Acting, AgentState::Idle), true);
        transition_rules.insert((AgentState::Acting, AgentState::Thinking), true);
        
        Self {
            current_state: initial_state.clone(),
            state_history: vec![(initial_state, chrono::Utc::now())],
            transition_rules,
        }
    }
    
    pub fn can_transition(&self, from: AgentState, to: AgentState) -> bool {
        self.transition_rules.get(&(from, to)).copied().unwrap_or(false)
    }
    
    pub fn transition_to(&mut self, new_state: AgentState) -> Result<()> {
        if self.can_transition(self.current_state.clone(), new_state.clone()) {
            self.current_state = new_state.clone();
            self.state_history.push((new_state, chrono::Utc::now()));
            Ok(())
        } else {
            Err(anyhow::anyhow!(
                "Invalid state transition from {:?} to {:?}",
                self.current_state,
                new_state
            ))
        }
    }
    
    pub fn get_state_history(&self) -> &[(AgentState, chrono::DateTime<chrono::Utc>)] {
        &self.state_history
    }
    
    pub async fn run_agent_cycle<A: Agent>(
        &self,
        agent: &mut A,
        state_machine: &mut StateMachine,
        max_cycles: Option<u32>,
    ) -> Result<()> {
        let mut cycle_count = 0;
        
        loop {
            if let Some(max) = max_cycles {
                if cycle_count >= max {
                    break;
                }
            }
            
            match state_machine.current_state {
                AgentState::Idle => {
                    // Check for pending tasks or messages
                    // Transition to Thinking if there's work to do
                    if Self::has_pending_work(agent).await {
                        state_machine.transition_to(AgentState::Thinking)?;
                    } else {
                        sleep(Duration::from_millis(100)).await;
                    }
                }
                
                AgentState::Thinking => {
                    // Process information and make decisions
                    let context = Self::build_context(agent).await?;
                    let decision = agent.make_decision(&context).await?;
                    
                    // Based on decision, either act or return to idle
                    if self.should_act(&decision) {
                        state_machine.transition_to(AgentState::Acting)?;
                    } else {
                        state_machine.transition_to(AgentState::Idle)?;
                    }
                }
                
                AgentState::Acting => {
                    // Execute actions based on decisions
                    let actions = Self::extract_actions(agent).await?;
                    for (action, params) in actions {
                        agent.execute_action(&action, &params).await?;
                    }
                    
                    // Return to thinking to evaluate results
                    state_machine.transition_to(AgentState::Thinking)?;
                }
            }
            
            cycle_count += 1;
        }
        
        Ok(())
    }
    
    async fn has_pending_work<A: Agent>(agent: &A) -> bool {
        // This would be implemented based on agent's current state
        // For now, return false to be implemented by concrete agents
        false
    }
    
    async fn build_context<A: Agent>(agent: &A) -> Result<HashMap<String, serde_json::Value>> {
        let mut context = HashMap::new();
        context.insert("agent_id".to_string(), serde_json::Value::String(agent.id().to_string()));
        context.insert("agent_name".to_string(), serde_json::Value::String(agent.name().to_string()));
        context.insert("state".to_string(), serde_json::Value::String(format!("{:?}", agent.state())));
        Ok(context)
    }
    
    fn should_act(&self, decision: &serde_json::Value) -> bool {
        decision.get("should_act")
            .and_then(|v| v.as_bool())
            .unwrap_or(false)
    }
    
    async fn extract_actions<A: Agent>(agent: &A) -> Result<Vec<(String, HashMap<String, serde_json::Value>)>> {
        // This would be implemented based on agent's decision
        // For now, return empty vector
        Ok(Vec::new())
    }
}
