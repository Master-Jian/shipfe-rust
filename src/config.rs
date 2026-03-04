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
    pub hashed_asset_patterns: Option<Vec<String>>,
    pub enable_shared: bool,
    pub keep_releases: u32,
    pub delete_old: bool,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct EnvironmentConfig {
    pub build_command: Option<String>,
    pub local_dist_path: String,
    pub servers: Vec<ServerConfig>,
    pub remote_tmp: String,
    pub sub_environments: Option<HashMap<String, SubEnvironmentConfig>>,
    pub hashed_asset_patterns: Option<Vec<String>>,
    pub enable_shared: Option<bool>,
    pub keep_releases: Option<u32>,
    pub delete_old: Option<bool>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct GlobalConfig {
    pub environments: HashMap<String, EnvironmentConfig>,
}

pub fn create_default_config() -> Result<(), crate::AppError> {
    let mut environments = HashMap::new();

    // Development environment
    environments.insert("dev".to_string(), EnvironmentConfig {
        build_command: Some("npm run build".to_string()),
        local_dist_path: "./dist".to_string(),
        servers: vec![ServerConfig {
            host: "dev.example.com".to_string(),
            port: 22,
            username: "deploy".to_string(),
            password: None,
            key_path: Some("~/.ssh/id_rsa".to_string()),
            remote_deploy_path: "/var/www/dev".to_string(),
        }],
        remote_tmp: "/tmp".to_string(),
        sub_environments: Some({
            let mut subs = HashMap::new();
            subs.insert("admin".to_string(), SubEnvironmentConfig {
                build_command: Some("npm run build:admin".to_string()),
                local_dist_path: None,
                remote_deploy_path: "/var/www/dev/admin".to_string(),
            });
            subs
        }),
        hashed_asset_patterns: Some(vec!["**/*.js".to_string(), "**/*.css".to_string()]),
        enable_shared: Some(false),
        keep_releases: Some(3),
        delete_old: Some(false),
    });

    // Production environment
    environments.insert("prod".to_string(), EnvironmentConfig {
        build_command: Some("npm run build".to_string()),
        local_dist_path: "./dist".to_string(),
        servers: vec![ServerConfig {
            host: "prod.example.com".to_string(),
            port: 22,
            username: "deploy".to_string(),
            password: None,
            key_path: Some("~/.ssh/prod_key".to_string()),
            remote_deploy_path: "/var/www/prod".to_string(),
        }],
        remote_tmp: "/tmp".to_string(),
        sub_environments: None,
        hashed_asset_patterns: Some(vec!["**/*.js".to_string(), "**/*.css".to_string(), "**/*.{png,jpg,svg}".to_string()]),
        enable_shared: Some(true),
        keep_releases: Some(10),
        delete_old: Some(false),
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