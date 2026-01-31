use crate::agent_framework::{Agent, AgentState, AgentType, AgentMessage, AgentError, AgentResult};
use crate::coordinator_agent::CoordinatorAgent;
use crate::editor_agent::EditorAgent;
use crate::terminal_agent::TerminalAgent;
use crate::agent_communication::{AgentCommunicationManager, CollaborationProtocol};
use crate::agent_decision_engine::{DecisionEngine, CollaborationPattern, PatternType, DecisionContext};
use crate::redis_coordination::RedisCoordinator;
use std::sync::{Arc, Mutex};
use tokio::signal;
use uuid::Uuid;

pub struct MultiAgentSystem {
    coordinator: Arc<Mutex<CoordinatorAgent>>,
    editor: Arc<Mutex<EditorAgent>>,
    terminal: Arc<Mutex<TerminalAgent>>,
    communication_manager: Arc<AgentCommunicationManager>,
    collaboration_protocol: Arc<CollaborationProtocol>,
    decision_engine: Arc<DecisionEngine>,
    redis_coordinator: Arc<RedisCoordinator>,
}

impl MultiAgentSystem {
    pub async fn new(redis_url: &str, workspace_path: String) -> AgentResult<Self> {
        log::info!("Initializing Multi-Agent System");
        
        let redis_coordinator = Arc::new(
            RedisCoordinator::new(redis_url, "multi_agent_system".to_string())
                .map_err(|e| AgentError::CommunicationError(e.to_string()))?
        );
        
        let coordinator = Arc::new(Mutex::new(
            CoordinatorAgent::new(redis_url)
                .map_err(|e| AgentError::CommunicationError(e.to_string()))?
        ));
        
        let editor = Arc::new(Mutex::new(
            EditorAgent::new(redis_url, workspace_path.clone())
                .map_err(|e| AgentError::CommunicationError(e.to_string()))?
        ));
        
        let terminal = Arc::new(Mutex::new(
            TerminalAgent::new(redis_url, workspace_path)
                .map_err(|e| AgentError::CommunicationError(e.to_string()))?
        ));
        
        let communication_manager = Arc::new(AgentCommunicationManager::new(redis_coordinator.clone()));
        let collaboration_protocol = Arc::new(CollaborationProtocol::new(communication_manager.clone()));
        let decision_engine = Arc::new(DecisionEngine::new());
        
        Ok(Self {
            coordinator,
            editor,
            terminal,
            communication_manager,
            collaboration_protocol,
            decision_engine,
            redis_coordinator,
        })
    }
    
    pub async fn initialize(&mut self) -> AgentResult<()> {
        log::info!("Initializing agents and communication");
        
        let coordinator_id = self.coordinator.lock().unwrap().agent_id().to_string();
        let editor_id = self.editor.lock().unwrap().agent_id().to_string();
        let terminal_id = self.terminal.lock().unwrap().agent_id().to_string();
        
        self.communication_manager.register_agent(&coordinator_id, "coordinator").await?;
        self.communication_manager.register_agent(&editor_id, "editor").await?;
        self.communication_manager.register_agent(&terminal_id, "terminal").await?;
        
        log::info!("All agents registered successfully");
        
        Ok(())
    }
    
    pub async fn start(&mut self) -> AgentResult<()> {
        log::info!("Starting Multi-Agent System");
        
        self.initialize().await?;
        
        let coordinator_id = self.coordinator.lock().unwrap().agent_id().to_string();
        let editor_id = self.editor.lock().unwrap().agent_id().to_string();
        let terminal_id = self.terminal.lock().unwrap().agent_id().to_string();
        
        let coordinator = self.coordinator.clone();
        let editor = self.editor.clone();
        let terminal = self.terminal.clone();
        
        // Clone comm_manager for each spawn
        let comm_manager1 = self.communication_manager.clone();
        let comm_manager2 = self.communication_manager.clone();
        let comm_manager3 = self.communication_manager.clone();
        
        tokio::spawn(async move {
            comm_manager1.start_message_listener(&coordinator_id, coordinator).await.unwrap();
        });
        
        tokio::spawn(async move {
            comm_manager2.start_message_listener(&editor_id, editor).await.unwrap();
        });
        
        tokio::spawn(async move {
            comm_manager3.start_message_listener(&terminal_id, terminal).await.unwrap();
        });
        
        log::info!("Multi-Agent System started successfully");
        
        Ok(())
    }
    
    pub async fn submit_task(&mut self, description: String) -> AgentResult<String> {
        log::info!("Submitting task: {}", description);
        
        let mut coordinator = self.coordinator.lock().unwrap();
        let task_id = coordinator.receive_task(description)?;
        
        Ok(task_id)
    }
    
    pub async fn process_tasks(&mut self) -> AgentResult<()> {
        log::info!("Processing tasks");
        
        let mut coordinator = self.coordinator.lock().unwrap();
        coordinator.process()?;
        
        Ok(())
    }
    
    pub async fn shutdown(&mut self) -> AgentResult<()> {
        log::info!("Shutting down Multi-Agent System");
        
        self.coordinator.lock().unwrap().shutdown()?;
        self.editor.lock().unwrap().shutdown()?;
        self.terminal.lock().unwrap().shutdown()?;
        
        log::info!("Multi-Agent System shut down successfully");
        
        Ok(())
    }
    
    pub async fn run_demo(&mut self) -> AgentResult<()> {
        log::info!("Running Multi-Agent System Demo");
        
        self.start().await?;
        
        let task_id = self.submit_task(
            "Create a simple Rust program that prints 'Hello, World!'".to_string()
        ).await?;
        
        log::info!("Task submitted with ID: {}", task_id);
        
        tokio::time::sleep(tokio::time::Duration::from_secs(5)).await;
        
        self.process_tasks().await?;
        
        tokio::time::sleep(tokio::time::Duration::from_secs(10)).await;
        
        self.shutdown().await?;
        
        Ok(())
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    env_logger::init();
    
    log::info!("=== Agent 7: Multi-Agent System Engineer ===");
    log::info!("Starting Multi-Agent System implementation");
    
    let mut system = MultiAgentSystem::new("redis://127.0.0.1:6379", "d:/MR.Darkpromth".to_string()).await?;
    
    system.run_demo().await?;
    
    log::info!("Demo completed successfully");
    
    Ok(())
}
