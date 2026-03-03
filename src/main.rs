mod license;
mod config;
mod deploy;

use clap::{Parser, Subcommand};
use license::{Capability, LicenseCtx, LicenseError};
use config::create_default_config;
use deploy::deploy_free;

#[derive(Parser)]
#[command(name = "shipfe", version)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    Init {},

    Activate {
        #[arg(long, default_value = "default")]
        profile: String,

        #[arg(long)]
        file: String,
    },

    Deploy {
        #[arg(long, default_value = "default")]
        profile: String,

        #[arg(long)]
        config: Option<String>,

        #[arg(long, default_value_t = false)]
        atomic: bool,

        #[arg(long, default_value_t = false)]
        rollback_on_fail: bool,

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
fn run() -> Result<(), LicenseError> {
    let cli = Cli::parse();

    match cli.command {
        Commands::Init {} => {
            create_default_config()?;
            println!("Initialized shipfe project with default config");
            Ok(())
        }

        Commands::Activate { profile, file } => {
            let raw = std::fs::read_to_string(&file)
                .map_err(|e| LicenseError::Invalid(e.to_string()))?;
            license::save_license_file(&profile, &raw)?;
            println!("Activated license for profile={profile}");
            Ok(())
        }

        Commands::Deploy {
            profile,
            config,
            atomic,
            rollback_on_fail,
            all_sub,
        } => {
            let lic = LicenseCtx::from_file_or_free(&profile)?;

            if atomic {
                lic.require(Capability::AtomicSwitch)?;
            }
            if rollback_on_fail {
                lic.require(Capability::Rollback)?;
            }

            let config_path = config.unwrap_or_else(|| "shipfe.config.json".to_string());

            let config_raw = std::fs::read_to_string(&config_path).map_err(|e| {
                LicenseError::Invalid(format!("Failed to read config {}: {}", config_path, e))
            })?;

            let global_config: crate::config::GlobalConfig = serde_json::from_str(&config_raw)
                .map_err(|e| LicenseError::Invalid(format!("config parse error: {e}")))?;

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
                    LicenseError::Invalid(format!(
                        "Environment '{}' not found in config",
                        base_profile
                    ))
                })?;

            if all_sub && env_config.sub_environments.is_some() && sub.is_none() {
                // 部署所有子环境
                if let Some(sub_envs) = &env_config.sub_environments {
                    for (sub_name, sub_config) in sub_envs {
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
                        };

                        deploy_free(&deploy_config)?;
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
                            LicenseError::Invalid(format!(
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
                        }
                    } else {
                        return Err(LicenseError::Invalid(format!(
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
            let lic = LicenseCtx::from_file_or_free(&profile)?;
            lic.require(Capability::Rollback)?;
            println!("rollback profile={profile}, to={to:?}");
            Ok(())
        }
    }
}