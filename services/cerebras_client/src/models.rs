use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum Model {
    Llama3_3_70b,
    GptOss120b,
    Qwen3_32b,
}

impl Model {
    pub fn as_str(&self) -> &'static str {
        match self {
            Model::Llama3_3_70b => "llama-3.3-70b",
            Model::GptOss120b => "gpt-oss-120b",
            Model::Qwen3_32b => "qwen-3-32b",
        }
    }

    pub fn parse(value: &str) -> Option<Self> {
        match value.trim().to_lowercase().as_str() {
            "llama-3.3-70b" | "llama3.3-70b" | "llama-3.3" => Some(Model::Llama3_3_70b),
            "gpt-oss-120b" | "gpt-oss" | "gpt-oss-120" => Some(Model::GptOss120b),
            "qwen-3-32b" | "qwen3-32b" | "qwen-3" => Some(Model::Qwen3_32b),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum RequestIntent {
    Chat,
    Code,
    Reasoning,
}

impl RequestIntent {
    pub fn default_model(&self) -> Model {
        match self {
            RequestIntent::Chat => Model::Llama3_3_70b,
            RequestIntent::Code => Model::Qwen3_32b,
            RequestIntent::Reasoning => Model::GptOss120b,
        }
    }
}
