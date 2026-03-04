mod config;
mod deploy;

use clap::{Parser, Subcommand};
use thiserror::Error;
use config::create_default_config;
use deploy::deploy_free;

#[derive(Error, Debug)]
pub enum AppError {
    #[error("invalid: {0}")]
    Invalid(String),
}

#[derive(Parser)]
#[command(name = "shipfe", version)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    Init {},

    Deploy {
        #[arg(long, default_value = "default")]
        profile: String,

        #[arg(long)]
        config: Option<String>,

        #[arg(long, default_value_t = false)]
        all_sub: bool,
    },

    Rollback {
        #[arg(long, default_value = "default")]
        profile: String,

        #[arg(long)]
        to: Option<String>,
    },
}

fn main() {
    if let Err(e) = run() {
        eprintln!("Error: {e}");
        std::process::exit(1);
    }
}
fn run() -> Result<(), AppError> {
    let cli = Cli::parse();

    match cli.command {
        Commands::Init {} => {
            create_default_config()?;
            println!("Initialized shipfe project with default config");
            Ok(())
        }

        Commands::Deploy {
            profile,
            config,
            all_sub,
        } => {
            // All features are now free and open source
            let config_path = config.unwrap_or_else(|| "shipfe.config.json".to_string());

            let config_raw = std::fs::read_to_string(&config_path).map_err(|e| {
                AppError::Invalid(format!("Failed to read config {}: {}", config_path, e))
            })?;

            let global_config: crate::config::GlobalConfig = serde_json::from_str(&config_raw)
                .map_err(|e| AppError::Invalid(format!("config parse error: {e}")))?;

            // 支持 profile-sub 格式，如 dev-admin
            let (base_profile, sub) = if profile.contains('-') {
                let mut it = profile.splitn(2, '-');
                let p0 = it.next().unwrap_or("").to_string();
                let p1 = it.next().map(|s| s.to_string());
                (p0, p1)
            } else {
                (profile.clone(), None)
            };

            let env_config = global_config
                .environments
                .get(&base_profile)
                .ok_or_else(|| {
                    AppError::Invalid(format!(
                        "Environment '{}' not found in config",
                        base_profile
                    ))
                })?;

            if all_sub && env_config.sub_environments.is_some() && sub.is_none() {
                // 部署所有子环境（按字母顺序）
                if let Some(sub_envs) = &env_config.sub_environments {
                    let mut sub_names: Vec<&String> = sub_envs.keys().collect();
                    sub_names.sort();
                    
                    for sub_name in sub_names {
                        if let Some(sub_config) = sub_envs.get(sub_name) {
                            println!("Deploying sub-environment: {}-{}", base_profile, sub_name);

                            let mut servers = env_config.servers.clone();
                            if !servers.is_empty() {
                                servers[0].remote_deploy_path = sub_config.remote_deploy_path.clone();
                            }

                            let deploy_config = crate::config::DeployParams {
                                build_command: sub_config
                                    .build_command
                                    .clone()
                                    .or_else(|| env_config.build_command.clone()),
                                local_dist_path: sub_config
                                    .local_dist_path
                                    .clone()
                                    .unwrap_or_else(|| env_config.local_dist_path.clone()),
                                servers,
                                remote_tmp: env_config.remote_tmp.clone(),
                                hashed_asset_patterns: env_config.hashed_asset_patterns.clone(),
                                enable_shared: env_config.enable_shared.unwrap_or(false),
                                keep_releases: env_config.keep_releases.unwrap_or(5),
                            };

                            deploy_free(&deploy_config)?;
                        }
                    }
                }

                println!(
                    "deploy all sub-environments for profile={}, config={}",
                    base_profile, config_path
                );
                Ok(())
            } else {
                let final_config = if let Some(sub_name) = &sub {
                    if let Some(sub_envs) = &env_config.sub_environments {
                        let sub_config = sub_envs.get(sub_name).ok_or_else(|| {
                            AppError::Invalid(format!(
                                "Sub-environment '{}' not found in '{}'",
                                sub_name, base_profile
                            ))
                        })?;

                        let mut servers = env_config.servers.clone();
                        if !servers.is_empty() {
                            servers[0].remote_deploy_path = sub_config.remote_deploy_path.clone();
                        }

                        crate::config::DeployParams {
                            build_command: sub_config
                                .build_command
                                .clone()
                                .or_else(|| env_config.build_command.clone()),
                            local_dist_path: sub_config
                                .local_dist_path
                                .clone()
                                .unwrap_or_else(|| env_config.local_dist_path.clone()),
                            servers,
                            remote_tmp: env_config.remote_tmp.clone(),
                            hashed_asset_patterns: env_config.hashed_asset_patterns.clone(),
                            enable_shared: env_config.enable_shared.unwrap_or(false),
                            keep_releases: env_config.keep_releases.unwrap_or(5),
                        }
                    } else {
                        return Err(AppError::Invalid(format!(
                            "No sub-environments defined for '{}'",
                            base_profile
                        )));
                    }
                } else {
                    crate::config::DeployParams {
                        build_command: env_config.build_command.clone(),
                        local_dist_path: env_config.local_dist_path.clone(),
                        servers: env_config.servers.clone(),
                        remote_tmp: env_config.remote_tmp.clone(),
                        hashed_asset_patterns: env_config.hashed_asset_patterns.clone(),
                        enable_shared: env_config.enable_shared.unwrap_or(false),
                        keep_releases: env_config.keep_releases.unwrap_or(5),
                    }
                };

                deploy_free(&final_config)?;
                println!(
                    "deploy profile={}{}, config={}",
                    base_profile,
                    sub.as_ref()
                        .map(|s| format!("-{}", s))
                        .unwrap_or_default(),
                    config_path
                );
                Ok(())
            }
        }

        Commands::Rollback { profile, to } => {
            // All features are now free and open source
            let to_version = to.ok_or_else(|| AppError::Invalid("rollback requires --to parameter".to_string()))?;

            let config_path = "shipfe.config.json".to_string();
            let config_raw = std::fs::read_to_string(&config_path).map_err(|e| {
                AppError::Invalid(format!("Failed to read config {}: {}", config_path, e))
            })?;
            let global_config: crate::config::GlobalConfig = serde_json::from_str(&config_raw)
                .map_err(|e| AppError::Invalid(format!("config parse error: {e}")))?;

            let (base_profile, sub) = if profile.contains('-') {
                let mut it = profile.splitn(2, '-');
                let p0 = it.next().unwrap_or("").to_string();
                let p1 = it.next().map(|s| s.to_string());
                (p0, p1)
            } else {
                (profile.clone(), None)
            };

            let env_config = global_config
                .environments
                .get(&base_profile)
                .ok_or_else(|| {
                    AppError::Invalid(format!(
                        "Environment '{}' not found in config",
                        base_profile
                    ))
                })?;

            if let Some(sub_name) = &sub {
                if let Some(sub_envs) = &env_config.sub_environments {
                    let sub_config = sub_envs.get(sub_name).ok_or_else(|| {
                        AppError::Invalid(format!(
                            "Sub-environment '{}' not found in '{}'",
                            sub_name, base_profile
                        ))
                    })?;

                    let servers = env_config.servers.clone();
                    if !servers.is_empty() {
                        let server = &servers[0];
                        deploy::rollback_to_version(server, &sub_config.remote_deploy_path, &to_version)?;
                    }
                } else {
                    return Err(AppError::Invalid(format!(
                        "No sub-environments defined for '{}'",
                        base_profile
                    )));
                }
            } else {
                let servers = &env_config.servers;
                if !servers.is_empty() {
                    let server = &servers[0];
                    deploy::rollback_to_version(server, &server.remote_deploy_path, &to_version)?;
                }
            }

            println!("rollback profile={}{}, to={}",
                base_profile,
                sub.as_ref().map(|s| format!("-{}", s)).unwrap_or_default(),
                to_version
            );
            Ok(())
        }
    }
}