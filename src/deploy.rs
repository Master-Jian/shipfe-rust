use std::fs::{metadata, File, OpenOptions};
use std::io::{self, Read, Write};
use tar::Builder;
use flate2::Compression;
use flate2::write::GzEncoder;
use ssh2::Session;
use std::net::TcpStream;
use chrono::{DateTime, Utc};
use std::path::Path;

use crate::config::ServerConfig;

fn compress_shared_assets(dist_path: &str, hashed_assets: &[String], output_path: &str) -> Result<(), crate::AppError> {
    let file = File::create(output_path).map_err(|e| crate::AppError::Invalid(e.to_string()))?;
    let enc = GzEncoder::new(file, Compression::default());
    let mut tar = Builder::new(enc);

    for asset in hashed_assets {
        let asset_path = Path::new(dist_path).join(asset);
        if asset_path.exists() {
            let asset_name = Path::new(asset)
                .file_name()
                .unwrap()
                .to_string_lossy()
                .to_string();
            tar.append_path_with_name(&asset_path, &asset_name)
                .map_err(|e| crate::AppError::Invalid(e.to_string()))?;
        }
    }

    tar.finish().map_err(|e| crate::AppError::Invalid(e.to_string()))?;
    Ok(())
}

fn compress_dist(dist_path: &str, output_path: &str) -> Result<(), crate::AppError> {
    let file = File::create(output_path).map_err(|e| crate::AppError::Invalid(e.to_string()))?;
    let enc = GzEncoder::new(file, Compression::default());
    let mut tar = Builder::new(enc);

    tar.append_dir_all("dist", dist_path)
        .map_err(|e| crate::AppError::Invalid(e.to_string()))?;
    tar.finish().map_err(|e| crate::AppError::Invalid(e.to_string()))?;

    Ok(())
}

#[derive(serde::Serialize, serde::Deserialize)]
struct Snapshot {
    id: String,
    timestamp: String,
    files: Vec<String>,
    hashed_assets: Vec<String>,
}

fn log_message(message: &str) {
    let timestamp = Utc::now().format("%Y-%m-%d %H:%M:%S");
    let log_entry = format!("[{}] {}\n", timestamp, message);
    println!("{}", message);
    if let Ok(mut file) = OpenOptions::new().create(true).append(true).open("shipfe.log") {
        let _ = file.write_all(log_entry.as_bytes());
    }
}

fn generate_snapshot(dist_path: &str, id: &str, patterns: &Option<Vec<String>>) -> Result<(), crate::AppError> {
    let mut files = Vec::new();
    let mut hashed_assets = Vec::new();

    fn visit_dir(
        dir: &Path,
        base: &Path,
        files: &mut Vec<String>,
        hashed_assets: &mut Vec<String>,
        patterns: &Option<Vec<String>>,
    ) -> io::Result<()> {
        if dir.is_dir() {
            for entry in std::fs::read_dir(dir)? {
                let entry = entry?;
                let path = entry.path();
                if path.is_dir() {
                    visit_dir(&path, base, files, hashed_assets, patterns)?;
                } else {
                    let rel_path = path.strip_prefix(base).unwrap().to_string_lossy().to_string();
                    files.push(rel_path.clone());

                    // ✅ 优先按用户 patterns（glob）识别 hashed assets
                    if let Some(pats) = patterns {
                        for pat in pats {
                            if glob::Pattern::new(pat).map_or(false, |pattern| pattern.matches(&rel_path)) {
                                hashed_assets.push(rel_path.clone());
                                break;
                            }
                        }
                    } else {
                        // 默认检测：文件名包含 -hash.（hash>=6位字母数字）
                        if rel_path.contains('-') && rel_path.contains('.') {
                            let parts: Vec<&str> = rel_path.split('.').collect();
                            if parts.len() >= 2 {
                                let filename = parts[parts.len() - 2];
                                if let Some(dash_pos) = filename.rfind('-') {
                                    let after_dash = &filename[dash_pos + 1..];
                                    if after_dash.len() >= 6 && after_dash.chars().all(|c| c.is_alphanumeric()) {
                                        hashed_assets.push(rel_path);
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
        Ok(())
    }

    visit_dir(
        Path::new(dist_path),
        Path::new(dist_path),
        &mut files,
        &mut hashed_assets,
        patterns,
    )
    .map_err(|e| crate::AppError::Invalid(e.to_string()))?;

    let snapshot = Snapshot {
        id: id.to_string(),
        timestamp: Utc::now().to_rfc3339(),
        files,
        hashed_assets,
    };

    let snapshot_path = format!("{}/shipfe.snapshot.json", dist_path);
    let json = serde_json::to_string_pretty(&snapshot).map_err(|e| crate::AppError::Invalid(e.to_string()))?;
    std::fs::write(&snapshot_path, json).map_err(|e| crate::AppError::Invalid(e.to_string()))?;
    log_message(&format!("Generated snapshot at {}", snapshot_path));

    Ok(())
}

fn run_build_command(cmd: &str) -> Result<(), crate::AppError> {
    log_message(&format!("Running build command: {}", cmd));
    use std::process::Command;
    let output = Command::new("sh")
        .arg("-c")
        .arg(cmd)
        .output()
        .map_err(|e| {
            log_message(&format!("Build command execution failed: {}", e));
            crate::AppError::Invalid(format!("Failed to run build command: {}", e))
        })?;

    if output.status.success() {
        log_message("Build successful");
        if !output.stdout.is_empty() {
            log_message(&format!("Build output: {}", String::from_utf8_lossy(&output.stdout)));
        }
        Ok(())
    } else {
        log_message("Build failed");
        if !output.stderr.is_empty() {
            log_message(&format!("Build error: {}", String::from_utf8_lossy(&output.stderr)));
        }
        Err(crate::AppError::Invalid(format!("Build command failed: {}", cmd)))
    }
}

pub fn deploy_free(config: &crate::config::DeployParams) -> Result<(), crate::AppError> {
    if let Some(build_cmd) = &config.build_command {
        run_build_command(build_cmd)?;
    }

    let dist_metadata = metadata(&config.local_dist_path)
        .map_err(|e| crate::AppError::Invalid(format!("Failed to get file metadata: {}", e)))?;
    let dist_mtime = dist_metadata
        .modified()
        .map_err(|e| crate::AppError::Invalid(format!("Failed to get file mtime: {}", e)))?;
    let timestamp = DateTime::<Utc>::from(dist_mtime).format("%Y%m%d_%H%M%S").to_string();

    generate_snapshot(&config.local_dist_path, &timestamp, &config.hashed_asset_patterns)?;

    let local_snapshot_path = format!("{}/shipfe.snapshot.json", config.local_dist_path);
    let hashed_assets: Vec<String> = if config.enable_shared {
        if let Ok(content) = std::fs::read_to_string(&local_snapshot_path) {
            serde_json::from_str::<Snapshot>(&content).map(|s| s.hashed_assets).unwrap_or_default()
        } else {
            Vec::new()
        }
    } else {
        Vec::new()
    };

    let archive_path = "/tmp/dist.tar.gz";
    log_message(&format!("Compressing {} to {}", config.local_dist_path, archive_path));
    compress_dist(&config.local_dist_path, &archive_path)?;
    log_message("Compression completed");

    let shared_archive_path = "/tmp/shared_assets.tar.gz";
    if config.enable_shared && !hashed_assets.is_empty() {
        log_message(&format!("Compressing shared assets to {}", shared_archive_path));
        compress_shared_assets(&config.local_dist_path, &hashed_assets, &shared_archive_path)?;
        log_message("Shared assets compression completed");
    }

    for server in &config.servers {
        upload_and_deploy(
            server,
            &archive_path,
            &shared_archive_path,
            &hashed_assets,
            &server.remote_deploy_path,
            &config.remote_tmp,
            &timestamp,
            config.enable_shared,
            config.keep_releases,
            config.delete_old,
            &config.local_dist_path,
        )?;
    }

    log_message("Deployment completed successfully");
    Ok(())
}

fn upload_and_deploy(
    server: &ServerConfig,
    local_archive: &str,
    local_shared_archive: &str,
    hashed_assets: &[String],
    remote_deploy_path: &str,
    remote_tmp: &str,
    timestamp: &str,
    enable_shared: bool,
    keep_releases: u32,
    delete_old: bool,
    _local_dist_path: &str,
) -> Result<(), crate::AppError> {
    let deploy_path = format!("{}/releases", remote_deploy_path);

    log_message(&format!("Connecting to server {}:{}", server.host, server.port));
    let tcp = TcpStream::connect(format!("{}:{}", server.host, server.port))
        .map_err(|e| crate::AppError::Invalid(e.to_string()))?;

    let mut sess = Session::new().map_err(|e| crate::AppError::Invalid(e.to_string()))?;
    sess.set_tcp_stream(tcp);
    sess.handshake().map_err(|e| crate::AppError::Invalid(e.to_string()))?;

    let auth_success = if let Some(password) = &server.password {
        sess.userauth_password(&server.username, password).is_ok()
    } else if let Ok(private_key) = std::env::var("SSH_PRIVATE_KEY") {
        sess.userauth_pubkey_memory(&server.username, None, &private_key, None).is_ok()
    } else if let Some(key_path) = &server.key_path {
        sess.userauth_pubkey_file(&server.username, None, Path::new(key_path), None).is_ok()
    } else {
        false
    };

    if !auth_success {
        return Err(crate::AppError::Invalid("SSH authentication failed".to_string()));
    }

    // 上传 dist.tar.gz
    let remote_archive = format!("{}/dist.tar.gz", remote_tmp);
    let file_size = metadata(local_archive).map_err(|e| crate::AppError::Invalid(e.to_string()))?.len();
    let mut remote_file = sess
        .scp_send(Path::new(&remote_archive), 0o644, file_size, None)
        .map_err(|e| crate::AppError::Invalid(e.to_string()))?;
    let mut local_file = File::open(local_archive).map_err(|e| crate::AppError::Invalid(e.to_string()))?;
    io::copy(&mut local_file, &mut remote_file).map_err(|e| crate::AppError::Invalid(e.to_string()))?;

    // 上传 shared_assets + current_hashes.txt
    let remote_shared_archive = format!("{}/shared_assets.tar.gz", remote_tmp);
    let remote_hashes = format!("{}/current_hashes.txt", remote_tmp);

    if enable_shared && !hashed_assets.is_empty() {
        // shared_assets.tar.gz
        let shared_file_size = metadata(local_shared_archive).map_err(|e| crate::AppError::Invalid(e.to_string()))?.len();
        let mut remote_shared_file = sess
            .scp_send(Path::new(&remote_shared_archive), 0o644, shared_file_size, None)
            .map_err(|e| crate::AppError::Invalid(e.to_string()))?;
        let mut local_shared_file = File::open(local_shared_archive).map_err(|e| crate::AppError::Invalid(e.to_string()))?;
        io::copy(&mut local_shared_file, &mut remote_shared_file).map_err(|e| crate::AppError::Invalid(e.to_string()))?;

        // ✅ current_hashes.txt：写 basename（只文件名）
        let local_hashes_path = "/tmp/current_hashes.txt";
        let hash_lines = hashed_assets
            .iter()
            .filter_map(|p| Path::new(p).file_name().map(|s| s.to_string_lossy().to_string()))
            .collect::<Vec<_>>()
            .join("\n");

        std::fs::write(local_hashes_path, hash_lines).map_err(|e| crate::AppError::Invalid(e.to_string()))?;

        let hashes_size = metadata(local_hashes_path).map_err(|e| crate::AppError::Invalid(e.to_string()))?.len();
        let mut remote_hashes_file = sess
            .scp_send(Path::new(&remote_hashes), 0o644, hashes_size, None)
            .map_err(|e| crate::AppError::Invalid(e.to_string()))?;
        let mut local_hashes_file = File::open(local_hashes_path).map_err(|e| crate::AppError::Invalid(e.to_string()))?;
        io::copy(&mut local_hashes_file, &mut remote_hashes_file).map_err(|e| crate::AppError::Invalid(e.to_string()))?;
    }

    // 部署命令
    let mut commands = vec![format!("mkdir -p {}", deploy_path)];
    if enable_shared {
        commands.push(format!("mkdir -p {}/shared/assets", remote_deploy_path));
    }

    commands.push(format!("cd {} && mkdir -p {}", deploy_path, timestamp));
    commands.push(format!(
        "cd {} && tar -xzf {} -C {} --strip-components=1",
        deploy_path, remote_archive, timestamp
    ));

    if enable_shared && !hashed_assets.is_empty() {
        commands.push(format!("mkdir -p {}/releases/{}/assets", remote_deploy_path, timestamp));
        commands.push(format!(
            "cd {}/releases/{}/assets && tar -xzf {} --overwrite",
            remote_deploy_path, timestamp, remote_shared_archive
        ));
        
        // Move shared assets to shared/assets directory and create hard links
        // ✅ 把 assets 里的文件移动到 shared/assets，并在 assets/ 内创建硬链接（路径正确，不删 assets）
        commands.push(format!(
            "set -e; \
             rel=\"{d}/releases/{t}\"; \
             shared=\"{d}/shared/assets\"; \
             mkdir -p \"$shared\"; \
             if [ -d \"$rel/assets\" ]; then \
               cd \"$rel/assets\"; \
               for f in *; do \
                 [ -f \"$f\" ] || continue; \
                 sf=\"$shared/$f\"; \
                 if [ ! -f \"$sf\" ]; then \
                   mv -f \"$f\" \"$sf\"; \
                 else \
                   rm -f \"$f\"; \
                 fi; \
                 ln -f \"$sf\" \"$f\"; \
               done; \
             fi; \
             true",
            d = remote_deploy_path,
            t = timestamp
        ));
        
        // Remove the temporary assets directory
        commands.push(format!("rm -rf {}/releases/{}/assets", remote_deploy_path, timestamp));
    }

    commands.push(format!(
        "cd {} && ln -sfn releases/{} current",
        remote_deploy_path, timestamp
    ));

    // ✅ 临时调试命令
    if enable_shared && !hashed_assets.is_empty() {
        commands.push(format!("cd {} && whoami && id", remote_deploy_path));
        commands.push(format!("cd {} && ls -la shared/assets | head -n 20 || true", remote_deploy_path));
        commands.push(format!("ls -la {} && head -n 20 {} || true", remote_hashes, remote_hashes));
    }

    // ✅ 修复点：清理 shared/assets 时避免空输入导致 rm 失败；并使用固定整行匹配
    if enable_shared && !hashed_assets.is_empty() {
        commands.push(format!(
            "cd {d} && \
             if [ -d shared/assets ]; then \
               if [ -f {h} ]; then \
                 ls -1 shared/assets/ 2>/dev/null | grep -v -F -x -f {h} 2>/dev/null | \
                 while IFS= read -r f; do \
                   rm -f \"shared/assets/$f\" 2>/dev/null || true; \
                 done; \
               fi; \
             fi; \
             true",
            d = remote_deploy_path,
            h = remote_hashes
        ));
    }

    if delete_old {
        commands.push(format!(
            "cd {} && for dir in releases/*; do if [ \"$dir\" != \"releases/{}\" ]; then rm -rf \"$dir\"; fi; done",
            remote_deploy_path, timestamp
        ));
    } else {
        commands.push(format!(
            "cd {} && ls -t releases/ | tail -n +{} | xargs -r -I {{}} rm -rf releases/{{}}",
            remote_deploy_path,
            keep_releases + 1
        ));
    }

    commands.push(format!("rm -f {}", remote_archive));
    if enable_shared && !hashed_assets.is_empty() {
        commands.push(format!("rm -f {}", remote_shared_archive));
        commands.push(format!("rm -f {}", remote_hashes));
    }

    for cmd in commands {
        let mut channel = sess.channel_session().map_err(|e| crate::AppError::Invalid(e.to_string()))?;
        channel.exec(&cmd).map_err(|e| crate::AppError::Invalid(e.to_string()))?;

        // ✅ 关键：stderr 合并进 stdout
        channel.handle_extended_data(ssh2::ExtendedData::Merge)
            .map_err(|e| crate::AppError::Invalid(e.to_string()))?;

        let mut output = String::new();
        channel.read_to_string(&mut output).map_err(|e| crate::AppError::Invalid(e.to_string()))?;
        channel.wait_close().map_err(|e| crate::AppError::Invalid(e.to_string()))?;
        let status = channel.exit_status().map_err(|e| crate::AppError::Invalid(e.to_string()))?;

        if status != 0 {
            return Err(crate::AppError::Invalid(format!(
                "Command failed: {}\n---- remote output ----\n{}",
                cmd,
                output
            )));
        }
    }

    Ok(())
}

pub fn rollback_to_version(server: &ServerConfig, remote_deploy_path: &str, to_version: &str) -> Result<(), crate::AppError> {
    log_message(&format!("Connecting to server {}:{}", server.host, server.port));
    let tcp = TcpStream::connect(format!("{}:{}", server.host, server.port))
        .map_err(|e| crate::AppError::Invalid(e.to_string()))?;

    let mut sess = Session::new().map_err(|e| crate::AppError::Invalid(e.to_string()))?;
    sess.set_tcp_stream(tcp);
    sess.handshake().map_err(|e| crate::AppError::Invalid(e.to_string()))?;

    let auth_success = if let Some(password) = &server.password {
        sess.userauth_password(&server.username, password).is_ok()
    } else if let Ok(private_key) = std::env::var("SSH_PRIVATE_KEY") {
        sess.userauth_pubkey_memory(&server.username, None, &private_key, None).is_ok()
    } else if let Some(key_path) = &server.key_path {
        sess.userauth_pubkey_file(&server.username, None, Path::new(key_path), None).is_ok()
    } else {
        false
    };

    if !auth_success {
        return Err(crate::AppError::Invalid("SSH authentication failed".to_string()));
    }

    // Check if the target version exists
    let check_cmd = format!("test -d {}/releases/{}", remote_deploy_path, to_version);
    let mut channel = sess.channel_session().map_err(|e| crate::AppError::Invalid(e.to_string()))?;
    channel.exec(&check_cmd).map_err(|e| crate::AppError::Invalid(e.to_string()))?;
    let mut output = String::new();
    channel.read_to_string(&mut output).map_err(|e| crate::AppError::Invalid(e.to_string()))?;
    channel.wait_close().map_err(|e| crate::AppError::Invalid(e.to_string()))?;
    let status = channel.exit_status().map_err(|e| crate::AppError::Invalid(e.to_string()))?;
    if status != 0 {
        return Err(crate::AppError::Invalid(format!("Version {} does not exist on server", to_version)));
    }

    // Update the current symlink
    let rollback_cmd = format!("cd {} && ln -sfn releases/{} current", remote_deploy_path, to_version);
    let mut channel = sess.channel_session().map_err(|e| crate::AppError::Invalid(e.to_string()))?;
    channel.exec(&rollback_cmd).map_err(|e| crate::AppError::Invalid(e.to_string()))?;
    let mut output = String::new();
    channel.read_to_string(&mut output).map_err(|e| crate::AppError::Invalid(e.to_string()))?;
    channel.wait_close().map_err(|e| crate::AppError::Invalid(e.to_string()))?;
    let status = channel.exit_status().map_err(|e| crate::AppError::Invalid(e.to_string()))?;
    if status != 0 {
        return Err(crate::AppError::Invalid(format!("Failed to rollback to version {}: {}", to_version, output)));
    }

    log_message(&format!("Successfully rolled back to version {}", to_version));
    Ok(())
}