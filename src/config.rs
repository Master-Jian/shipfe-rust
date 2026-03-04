use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ServerConfig {
    pub host: String,
    pub port: u16,
    pub username: String,
    pub password: Option<String>,
    pub key_path: Option<String>,
    pub remote_deploy_path: String,
    pub delete_old: bool,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct SubEnvironmentConfig {
    pub build_command: Option<String>,
    pub local_dist_path: Option<String>,
    pub remote_deploy_path: String,
}

#[derive(Debug)]
pub struct DeployParams {
    pub build_command: Option<String>,
    pub local_dist_path: String,
    pub servers: Vec<ServerConfig>,
    pub remote_tmp: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct EnvironmentConfig {
    pub build_command: Option<String>,
    pub local_dist_path: String,
    pub servers: Vec<ServerConfig>,
    pub remote_tmp: String,
    pub sub_environments: Option<HashMap<String, SubEnvironmentConfig>>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct GlobalConfig {
    pub environments: HashMap<String, EnvironmentConfig>,
}

pub fn create_default_config() -> Result<(), crate::AppError> {
    let mut environments = HashMap::new();
    environments.insert("default".to_string(), EnvironmentConfig {
        build_command: Some("npm run build".to_string()),
        local_dist_path: "./dist".to_string(),
        servers: vec![ServerConfig {
            host: "localhost".to_string(),
            port: 22,
            username: "user".to_string(),
            password: Some("password".to_string()),
            key_path: None,
            remote_deploy_path: "/var/www".to_string(),
            delete_old: false,
        }],
        remote_tmp: "/tmp".to_string(),
        sub_environments: None,
    });

    let global_config = GlobalConfig { environments };

    let config_json = serde_json::to_string_pretty(&global_config)
        .map_err(|e| crate::AppError::Invalid(e.to_string()))?;

    std::fs::write("shipfe.config.json", config_json)
        .map_err(|e| crate::AppError::Invalid(e.to_string()))?;

    // Handle .gitignore file
    update_gitignore()?;

    Ok(())
}

fn update_gitignore() -> Result<(), crate::AppError> {
    let gitignore_path = ".gitignore";
    let entries_to_add = vec!["shipfe.log", "shipfe.config.json"];

    match std::fs::read_to_string(gitignore_path) {
        Ok(content) => {
            // .gitignore exists, check if entries are already there
            let mut lines: Vec<String> = content.lines().map(|s| s.to_string()).collect();
            let mut modified = false;

            for entry in &entries_to_add {
                if !lines.iter().any(|line| line.trim() == *entry) {
                    lines.push(entry.to_string());
                    modified = true;
                }
            }

            if modified {
                let new_content = lines.join("\n") + "\n";
                std::fs::write(gitignore_path, new_content)
                    .map_err(|e| crate::AppError::Invalid(e.to_string()))?;
                println!("Updated .gitignore with shipfe-related entries");
            }
        }
        Err(_) => {
            // .gitignore doesn't exist, create it with the entries
            let content = entries_to_add.join("\n") + "\n";
            std::fs::write(gitignore_path, content)
                .map_err(|e| crate::AppError::Invalid(e.to_string()))?;
            println!("Created .gitignore with shipfe-related entries");
        }
    }

    Ok(())
}