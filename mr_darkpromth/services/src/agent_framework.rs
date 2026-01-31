use serde::{Deserialize, Serialize};
use std::fmt;
use thiserror::Error;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum AgentState {
    Idle,
    Thinking,
    Acting,
    Error,
    Shutdown,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AgentType {
    Coordinator,
    Editor,
    Terminal,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentMessage {
    pub id: Uuid,
    pub from_agent: String,
    pub to_agent: String,
    pub message_type: String,
    pub content: serde_json::Value,
    pub timestamp: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Error)]
pub enum AgentError {
    #[error("Agent is in invalid state: {0:?}")]
    InvalidState(AgentState),
    #[error("Message processing failed: {0}")]
    MessageProcessingFailed(String),
    #[error("Action execution failed: {0}")]
    ActionExecutionFailed(String),
    #[error("Communication error: {0}")]
    CommunicationError(String),
    #[error("Serialization error: {0}")]
    SerializationError(String),
}

pub type AgentResult<T> = Result<T, AgentError>;

pub trait Agent: Send + Sync {
    fn agent_id(&self) -> &str;
    fn agent_type(&self) -> AgentType;
    fn state(&self) -> AgentState;
    
    fn set_state(&mut self, state: AgentState) -> AgentResult<()> {
        match (&self.state(), &state) {
            (AgentState::Shutdown, _) => Err(AgentError::InvalidState(AgentState::Shutdown)),
            (_, AgentState::Shutdown) => Ok(()),
            (AgentState::Error, AgentState::Idle) => Ok(()),
            (AgentState::Idle, AgentState::Thinking) => Ok(()),
            (AgentState::Thinking, AgentState::Acting) => Ok(()),
            (AgentState::Acting, AgentState::Idle) => Ok(()),
            (AgentState::Acting, AgentState::Thinking) => Ok(()),
            (AgentState::Thinking, AgentState::Idle) => Ok(()),
            (current, new) if current == new => Ok(()),
            (current, new) => Err(AgentError::InvalidState(current.clone())),
        }
    }
    
    fn handle_message(&mut self, message: AgentMessage) -> AgentResult<()>;
    fn process(&mut self) -> AgentResult<()>;
    fn shutdown(&mut self) -> AgentResult<()>;
}

pub struct AgentStateMachine {
    state: AgentState,
    history: Vec<(AgentState, chrono::DateTime<chrono::Utc>)>,
}

impl AgentStateMachine {
    pub fn new() -> Self {
        Self {
            state: AgentState::Idle,
            history: Vec::new(),
        }
    }
    
    pub fn current_state(&self) -> AgentState {
        self.state.clone()
    }
    
    pub fn transition_to(&mut self, new_state: AgentState) -> AgentResult<()> {
        self.history.push((self.state.clone(), chrono::Utc::now()));
        
        match (&self.state, &new_state) {
            (AgentState::Shutdown, _) => Err(AgentError::InvalidState(AgentState::Shutdown)),
            (_, AgentState::Shutdown) => {
                self.state = new_state;
                Ok(())
            }
            (AgentState::Error, AgentState::Idle) => {
                self.state = new_state;
                Ok(())
            }
            (AgentState::Idle, AgentState::Thinking) => {
                self.state = new_state;
                Ok(())
            }
            (AgentState::Thinking, AgentState::Acting) => {
                self.state = new_state;
                Ok(())
            }
            (AgentState::Acting, AgentState::Idle) => {
                self.state = new_state;
                Ok(())
            }
            (AgentState::Acting, AgentState::Thinking) => {
                self.state = new_state;
                Ok(())
            }
            (AgentState::Thinking, AgentState::Idle) => {
                self.state = new_state;
                Ok(())
            }
            (current, new) if current == new => Ok(()),
            (current, _) => Err(AgentError::InvalidState(current.clone())),
        }
    }
    
    pub fn history(&self) -> &[(AgentState, chrono::DateTime<chrono::Utc>)] {
        &self.history
    }
    
    pub fn state_duration(&self) -> chrono::Duration {
        if let Some((_, timestamp)) = self.history.last() {
            chrono::Utc::now() - *timestamp
        } else {
            chrono::Duration::zero()
        }
    }
}

impl Default for AgentStateMachine {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_state_transitions() {
        let mut machine = AgentStateMachine::new();
        assert_eq!(machine.current_state(), AgentState::Idle);
        
        assert!(machine.transition_to(AgentState::Thinking).is_ok());
        assert_eq!(machine.current_state(), AgentState::Thinking);
        
        assert!(machine.transition_to(AgentState::Acting).is_ok());
        assert_eq!(machine.current_state(), AgentState::Acting);
        
        assert!(machine.transition_to(AgentState::Idle).is_ok());
        assert_eq!(machine.current_state(), AgentState::Idle);
    }
    
    #[test]
    fn test_invalid_transitions() {
        let mut machine = AgentStateMachine::new();
        
        assert!(machine.transition_to(AgentState::Shutdown).is_ok());
        assert!(machine.transition_to(AgentState::Thinking).is_err());
    }
}
