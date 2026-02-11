// MR.DarkPromth AI-Driven Automated Documentation Service
// Phase 3: Advanced Integration

use std::path::{Path, PathBuf};
use serde::{Deserialize, Serialize};
use walkdir::WalkDir;
use crate::cerebras_integration::CerebrasClient;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DocModule {
    pub name: String,
    pub path: String,
    pub summary: String,
    pub functions: Vec<DocFunction>,
    pub structs: Vec<DocStruct>,
    pub enums: Vec<DocEnum>,
    pub dependencies: Vec<String>,
    pub last_updated: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DocFunction {
    pub name: String,
    pub signature: String,
    pub description: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DocStruct {
    pub name: String,
    pub description: String,
    pub fields: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DocEnum {
    pub name: String,
    pub description: String,
    pub variants: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DependencyGraph {
    pub nodes: Vec<String>,
    pub edges: Vec<(String, String)>,
}

pub struct AutoDocService {
    client: CerebrasClient,
    base_path: PathBuf,
}

impl AutoDocService {
    pub fn new(api_key: String, base_path: PathBuf) -> Self {
        Self {
            client: CerebrasClient::with_api_key(api_key),
            base_path,
        }
    }

    pub async fn scan_codebase(&self) -> Result<Vec<DocModule>, Box<dyn std::error::Error + Send + Sync>> {
        let mut modules = Vec::new();

        for entry in WalkDir::new(&self.base_path)
            .into_iter()
            .filter_map(|e| e.ok())
        {
            if entry.path().extension().is_none_or(|ext| ext != "rs") {
                continue;
            }
            let path = entry.path();
            let relative_path = path.strip_prefix(&self.base_path).map_err(|e| e.to_string())?.to_string_lossy().to_string();
            
            // Skip common excluded directories
            if relative_path.contains("target/") || relative_path.contains(".git/") {
                continue;
            }

            let content = std::fs::read_to_string(path)?;
            let module = self.generate_doc_for_file(&relative_path, &content).await?;
            modules.push(module);
        }

        Ok(modules)
    }

    async fn generate_doc_for_file(&self, path: &str, content: &str) -> Result<DocModule, Box<dyn std::error::Error + Send + Sync>> {
        // Extract dependencies using regex
        let dependencies = self.extract_dependencies(content);
        let mut module_name = path.split('/').next_back().unwrap_or(path).replace(".rs", "");
        if module_name == "mod" || module_name == "lib" || module_name == "main" {
            // Try to use parent directory name
            if let Some(parent) = std::path::Path::new(path).parent() {
                 if let Some(parent_name) = parent.file_name() {
                     module_name = parent_name.to_string_lossy().to_string();
                 }
            }
        }

        let prompt = format!(
            "Analyze the following Rust code and generate a JSON documentation summary.\n\
             Return ONLY a JSON object with the following structure:\n\
             {{\n\
               \"name\": \"module name\",\n\
               \"summary\": \"high-level summary\",\n\
               \"functions\": [{{\"name\": \"func\", \"signature\": \"impl\", \"description\": \"desc\"}}],\n\
               \"structs\": [{{\"name\": \"struct\", \"description\": \"desc\", \"fields\": [\"field1\"]}}],\n\
               \"enums\": [{{\"name\": \"enum\", \"description\": \"desc\", \"variants\": [\"var1\"]}}]\n\
             }}\n\n\
             CODE:\n{}",
            content
        );

        // Limit content size for LLM to avoid context limit errors
        let limited_content = if prompt.len() > 12000 {
            &prompt[..12000]
        } else {
            &prompt
        };

        // Try LLM generation
        let doc_result = match self.client.chat_completion(limited_content, None).await {
            Ok(response) => {
                // Basic JSON extraction (assuming LLM returns clean JSON or wraps in code blocks)
                let json_str = if let Some(start) = response.find('{') {
                    if let Some(end) = response.rfind('}') {
                        &response[start..=end]
                    } else {
                        &response
                    }
                } else {
                    &response
                };

                match serde_json::from_str::<DocModule>(json_str) {
                    Ok(mut m) => {
                        m.path = path.to_string();
                        m.dependencies = dependencies.clone();
                        m.last_updated = chrono::Utc::now();
                        Ok(m)
                    },
                    Err(e) => {
                        log::error!("Failed to parse LLM JSON for {}: {}. Fallback to basic parsing.", path, e);
                        Err(Box::new(e) as Box<dyn std::error::Error + Send + Sync>)
                    }
                }
            },
            Err(e) => {
                log::error!("LLM request failed for {}: {}. Fallback to basic parsing.", path, e);
                Err(Box::new(std::io::Error::other(e.to_string())) as Box<dyn std::error::Error + Send + Sync>)
            }
        };

        // Fallback or Return Success
        match doc_result {
            Ok(module) => Ok(module),
            Err(_) => {
                // Fallback: manually scrape basic info using regex
                log::info!("Using fallback documentation generation for {}", path);
                let functions = self.extract_functions_regex(content);
                let structs = self.extract_structs_regex(content);
                let enums = self.extract_enums_regex(content);
                
                Ok(DocModule {
                    name: module_name,
                    path: path.to_string(),
                    summary: "Auto-generated documentation (Fallback mode)".to_string(),
                    functions,
                    structs,
                    enums,
                    dependencies,
                    last_updated: chrono::Utc::now(),
                })
            }
        }
    }

    fn extract_functions_regex(&self, content: &str) -> Vec<DocFunction> {
        let mut functions = Vec::new();
        let lines: Vec<&str> = content.lines().collect();

        for (i, line) in lines.iter().enumerate() {
            let trimmed = line.trim();

            // Match function declarations: pub/pub(crate)/async fn name(...)
            if let Some(fn_pos) = trimmed.find("fn ") {
                // Skip if inside a comment
                if trimmed.starts_with("//") || trimmed.starts_with("*") {
                    continue;
                }

                // Extract function name
                let after_fn = &trimmed[fn_pos + 3..];
                let name: String = after_fn.chars().take_while(|c| c.is_alphanumeric() || *c == '_').collect();
                if name.is_empty() || name == "main" {
                    continue;
                }

                // Build full signature from the line
                let signature = trimmed.trim_end_matches('{').trim().to_string();

                // Collect preceding doc comments (/// lines)
                let mut doc_lines = Vec::new();
                let mut j = i;
                while j > 0 {
                    j -= 1;
                    let prev = lines[j].trim();
                    if prev.starts_with("///") {
                        doc_lines.push(prev.trim_start_matches("///").trim().to_string());
                    } else if prev.starts_with("#[") || prev.is_empty() {
                        // Skip attributes and blank lines
                        continue;
                    } else {
                        break;
                    }
                }

                doc_lines.reverse();
                let description = if doc_lines.is_empty() {
                    "Undocumented function".to_string()
                } else {
                    doc_lines.join(" ")
                };

                functions.push(DocFunction { name, signature, description });
            }
        }
        functions
    }

    fn extract_structs_regex(&self, content: &str) -> Vec<DocStruct> {
        let mut structs = Vec::new();
        let lines: Vec<&str> = content.lines().collect();

        for (i, line) in lines.iter().enumerate() {
            let trimmed = line.trim();
            if trimmed.starts_with("//") || trimmed.starts_with("*") {
                continue;
            }

            // Match: pub struct Name or struct Name
            if let Some(pos) = trimmed.find("struct ") {
                // Skip "pub(crate) struct" detection — we still match it
                let after = &trimmed[pos + 7..];
                let name: String = after.chars().take_while(|c| c.is_alphanumeric() || *c == '_').collect();
                if name.is_empty() {
                    continue;
                }

                // Collect doc comments
                let mut doc_lines = Vec::new();
                let mut j = i;
                while j > 0 {
                    j -= 1;
                    let prev = lines[j].trim();
                    if prev.starts_with("///") {
                        doc_lines.push(prev.trim_start_matches("///").trim().to_string());
                    } else if prev.starts_with("#[") || prev.is_empty() {
                        continue;
                    } else {
                        break;
                    }
                }
                doc_lines.reverse();

                // Extract #[derive(...)] attributes
                let mut derives = Vec::new();
                let mut k = i;
                while k > 0 {
                    k -= 1;
                    let prev = lines[k].trim();
                    if prev.starts_with("#[derive(") {
                        let inner = prev.trim_start_matches("#[derive(").trim_end_matches(")]");
                        derives.push(format!("#[derive({})]", inner));
                        break;
                    } else if prev.starts_with("///") || prev.is_empty() {
                        continue;
                    } else {
                        break;
                    }
                }

                // Extract fields (scan until closing brace)
                let mut fields = Vec::new();
                if trimmed.ends_with('{') || (i + 1 < lines.len() && lines[i + 1].trim() == "{") {
                    let start = if trimmed.ends_with('{') { i + 1 } else { i + 2 };
                    for fl in lines.iter().skip(start) {
                        let fl = fl.trim();
                        if fl == "}" {
                            break;
                        }
                        if fl.starts_with("pub ") || (!fl.starts_with("//") && fl.contains(':')) {
                            let field = fl.trim_end_matches(',').to_string();
                            fields.push(field);
                        }
                    }
                }

                let description = if !doc_lines.is_empty() {
                    doc_lines.join(" ")
                } else if !derives.is_empty() {
                    format!("Struct with {}", derives.join(", "))
                } else {
                    "Undocumented struct".to_string()
                };

                structs.push(DocStruct { name, description, fields });
            }
        }
        structs
    }

    fn extract_enums_regex(&self, content: &str) -> Vec<DocEnum> {
        let mut enums = Vec::new();
        let lines: Vec<&str> = content.lines().collect();

        for (i, line) in lines.iter().enumerate() {
            let trimmed = line.trim();
            if trimmed.starts_with("//") || trimmed.starts_with("*") {
                continue;
            }

            // Match: pub enum Name or enum Name
            if let Some(pos) = trimmed.find("enum ") {
                let after = &trimmed[pos + 5..];
                let name: String = after.chars().take_while(|c| c.is_alphanumeric() || *c == '_').collect();
                if name.is_empty() {
                    continue;
                }

                // Collect doc comments
                let mut doc_lines = Vec::new();
                let mut j = i;
                while j > 0 {
                    j -= 1;
                    let prev = lines[j].trim();
                    if prev.starts_with("///") {
                        doc_lines.push(prev.trim_start_matches("///").trim().to_string());
                    } else if prev.starts_with("#[") || prev.is_empty() {
                        continue;
                    } else {
                        break;
                    }
                }
                doc_lines.reverse();

                // Extract variants
                let mut variants = Vec::new();
                if trimmed.ends_with('{') || (i + 1 < lines.len() && lines[i + 1].trim() == "{") {
                    let start = if trimmed.ends_with('{') { i + 1 } else { i + 2 };
                    for vl in lines.iter().skip(start) {
                        let vl = vl.trim();
                        if vl == "}" {
                            break;
                        }
                        if !vl.starts_with("//") && !vl.is_empty() {
                            let variant = vl.trim_end_matches(',').to_string();
                            variants.push(variant);
                        }
                    }
                }

                let description = if doc_lines.is_empty() {
                    "Undocumented enum".to_string()
                } else {
                    doc_lines.join(" ")
                };

                enums.push(DocEnum { name, description, variants });
            }
        }
        enums
    }

    fn extract_dependencies(&self, content: &str) -> Vec<String> {
        let re = regex::Regex::new(r"use\s+([^;]+);").unwrap();
        let mut deps = Vec::new();
        
        for cap in re.captures_iter(content) {
            if let Some(dep) = cap.get(1) {
                let dep_str = dep.as_str().trim();
                // Extract base crate/module name
                let base = dep_str.split("::").next().unwrap_or(dep_str);
                if !deps.contains(&base.to_string()) && base != "crate" && base != "super" && base != "self" && base != "std" {
                    deps.push(base.to_string());
                }
            }
        }
        deps
    }

    pub fn generate_dependency_graph(&self, modules: &[DocModule]) -> DependencyGraph {
        let mut nodes = Vec::new();
        let mut edges = Vec::new();

        for module in modules {
            if !nodes.contains(&module.name) {
                nodes.push(module.name.clone());
            }

            for dep in &module.dependencies {
                if !nodes.contains(dep) {
                    nodes.push(dep.clone());
                }
                edges.push((module.name.clone(), dep.clone()));
            }
        }

        DependencyGraph { nodes, edges }
    }

    pub async fn save_docs(&self, modules: &[DocModule], output_dir: &Path) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        if !output_dir.exists() {
            std::fs::create_dir_all(output_dir)?;
        }

        for module in modules {
            let file_name = module.path.replace("/", "_").replace(".rs", ".json");
            let file_path = output_dir.join(file_name);
            let json = serde_json::to_string_pretty(module)?;
            std::fs::write(file_path, json)?;
        }

        // Generate and save graph
        let graph = self.generate_dependency_graph(modules);
        let graph_json = serde_json::to_string_pretty(&graph)?;
        std::fs::write(output_dir.join("dependency_graph.json"), graph_json)?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_extract_dependencies() {
        let service = AutoDocService::new("test_key".to_string(), PathBuf::from("."));
        let content = r#"
            use std::collections::HashMap;
            use serde::{Deserialize, Serialize};
            use crate::some_module::SomeStruct;
            use tokio::time::Duration;
            use external_crate::prelude::*;
        "#;

        let deps = service.extract_dependencies(content);
        assert!(deps.contains(&"serde".to_string()));
        assert!(deps.contains(&"tokio".to_string()));
        assert!(deps.contains(&"external_crate".to_string()));
        // crate, std should be filtered
        assert!(!deps.contains(&"crate".to_string()));
        assert!(!deps.contains(&"std".to_string()));
    }

    #[tokio::test]
    async fn test_dependency_graph() {
        let service = AutoDocService::new("test_key".to_string(), PathBuf::from("."));
        
        let mod1 = DocModule {
            name: "mod1".to_string(),
            path: "src/mod1.rs".to_string(),
            summary: "Module 1".to_string(),
            functions: vec![],
            structs: vec![],
            enums: vec![],
            dependencies: vec!["mod2".to_string(), "serde".to_string()],
            last_updated: chrono::Utc::now(),
        };

        let mod2 = DocModule {
            name: "mod2".to_string(),
            path: "src/mod2.rs".to_string(),
            summary: "Module 2".to_string(),
            functions: vec![],
            structs: vec![],
            enums: vec![],
            dependencies: vec![],
            last_updated: chrono::Utc::now(),
        };

        let graph = service.generate_dependency_graph(&[mod1, mod2]);
        
        // Check nodes
        assert!(graph.nodes.contains(&"mod1".to_string()));
        assert!(graph.nodes.contains(&"mod2".to_string()));
        assert!(graph.nodes.contains(&"serde".to_string()));
        
        // Check edges
        assert!(graph.edges.contains(&("mod1".to_string(), "mod2".to_string())));
        assert!(graph.edges.contains(&("mod1".to_string(), "serde".to_string())));
    }
}
