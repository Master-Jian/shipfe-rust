use std::fs::{metadata, File, OpenOptions};
use std::io::{self, Read, Write};
use tar::Builder;
use flate2::Compression;
use flate2::write::GzEncoder;
use ssh2::Session;
use std::net::TcpStream;
use chrono::Utc;
use std::path::Path;

use crate::config::ServerConfig;

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

fn write_hashed_assets_manifest(dist_path: &str, hashed_assets: &[String]) -> Result<(), crate::AppError> {
    let manifest_path = format!("{}/shipfe.hashed_assets.txt", dist_path);
    let content = if hashed_assets.is_empty() {
        String::new()
    } else {
        format!("{}\n", hashed_assets.join("\n"))
    };

    std::fs::write(&manifest_path, content).map_err(|e| crate::AppError::Invalid(e.to_string()))?;
    Ok(())
}

pub fn deploy_free(config: &crate::config::DeployParams) -> Result<(), crate::AppError> {
    if let Some(build_cmd) = &config.build_command {
        run_build_command(build_cmd)?;
    }

    // 使用当前时间作为部署版本号，避免依赖 dist 目录 mtime 导致时间戳滞后
    let timestamp = Utc::now().format("%Y%m%d_%H%M%S").to_string();

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

    write_hashed_assets_manifest(&config.local_dist_path, &hashed_assets)?;

    let archive_path = "/tmp/dist.tar.gz";
    log_message(&format!("Compressing {} to {}", config.local_dist_path, archive_path));
    compress_dist(&config.local_dist_path, &archive_path)?;
    log_message("Compression completed");

    for server in &config.servers {
        upload_and_deploy(
            server,
            &archive_path,
            &hashed_assets,
            &server.remote_deploy_path,
            &config.remote_tmp,
            &timestamp,
            config.enable_shared,
            config.keep_releases,
            &config.local_dist_path,
        )?;
    }

    log_message("Deployment completed successfully");
    Ok(())
}

fn upload_and_deploy(
    server: &ServerConfig,
    local_archive: &str,
    hashed_assets: &[String],
    remote_deploy_path: &str,
    remote_tmp: &str,
    timestamp: &str,
    enable_shared: bool,
    keep_releases: u32,
    _local_dist_path: &str,
) -> Result<(), crate::AppError> {
    let deploy_path = format!("{}/releases", remote_deploy_path);

    log_message(&format!("Connecting to server {}:{}", server.host, server.port));
    let tcp = TcpStream::connect(format!("{}:{}", server.host, server.port))
        .map_err(|e| crate::AppError::Invalid(e.to_string()))?;

    let mut sess = Session::new().map_err(|e| crate::AppError::Invalid(e.to_string()))?;
    sess.set_tcp_stream(tcp);
    sess.handshake()
        .map_err(|e| crate::AppError::Invalid(e.to_string()))?;

    let auth_success = if let Some(password) = &server.password {
        sess.userauth_password(&server.username, password).is_ok()
    } else if let Ok(private_key) = std::env::var("SSH_PRIVATE_KEY") {
        sess.userauth_pubkey_memory(&server.username, None, &private_key, None)
            .is_ok()
    } else if let Some(key_path) = &server.key_path {
        sess.userauth_pubkey_file(&server.username, None, Path::new(key_path), None)
            .is_ok()
    } else {
        false
    };

    if !auth_success {
        return Err(crate::AppError::Invalid(
            "SSH authentication failed".to_string(),
        ));
    }

    // 1) 上传 dist.tar.gz 到远端临时目录
    let remote_archive = format!("{}/dist.tar.gz", remote_tmp);
    let file_size = metadata(local_archive)
        .map_err(|e| crate::AppError::Invalid(e.to_string()))?
        .len();

    let mut remote_file = sess
        .scp_send(Path::new(&remote_archive), 0o644, file_size, None)
        .map_err(|e| crate::AppError::Invalid(e.to_string()))?;

    let mut local_file =
        File::open(local_archive).map_err(|e| crate::AppError::Invalid(e.to_string()))?;
    io::copy(&mut local_file, &mut remote_file)
        .map_err(|e| crate::AppError::Invalid(e.to_string()))?;

    // 2) 如果启用 shared，则上传当前发布的 hash 资源清单
    let remote_hashes = format!("{}/current_hashes.txt", remote_tmp);

    if enable_shared && !hashed_assets.is_empty() {
        let local_hashes_path = "/tmp/current_hashes.txt";
        let hash_lines = hashed_assets.join("\n");

        std::fs::write(local_hashes_path, hash_lines)
            .map_err(|e| crate::AppError::Invalid(e.to_string()))?;

        let hashes_size = metadata(local_hashes_path)
            .map_err(|e| crate::AppError::Invalid(e.to_string()))?
            .len();

        let mut remote_hashes_file = sess
            .scp_send(Path::new(&remote_hashes), 0o644, hashes_size, None)
            .map_err(|e| crate::AppError::Invalid(e.to_string()))?;

        let mut local_hashes_file =
            File::open(local_hashes_path).map_err(|e| crate::AppError::Invalid(e.to_string()))?;

        io::copy(&mut local_hashes_file, &mut remote_hashes_file)
            .map_err(|e| crate::AppError::Invalid(e.to_string()))?;
    }

    let mut commands = vec![];

    // 3) 基础目录准备
    commands.push(format!("mkdir -p {}", deploy_path));
    if enable_shared {
        commands.push(format!("mkdir -p {}/shared", remote_deploy_path));
    }

    // 4) 创建本次 release 目录并解压
    commands.push(format!("cd {} && mkdir -p {}", deploy_path, timestamp));
    commands.push(format!(
        "cd {} && tar -xzf {} -C {} --strip-components=1",
        deploy_path, remote_archive, timestamp
    ));

    // 5) 如果启用 shared：将当前 release 中的 hash 资源移动到 shared，并在原位置创建硬链接
    if enable_shared && !hashed_assets.is_empty() {
    commands.push(format!(
        r#"set -e;
rel_root="{d}/releases/{t}";
shared_root="{d}/shared";
hashes="{h}";
manifest="$rel_root/shipfe.hashed_assets.txt";

mkdir -p "$shared_root";

if [ -f "$hashes" ]; then
    cp "$hashes" "$manifest";

    moved=0
    linked=0
    skipped=0

    while IFS= read -r p; do
        [ -z "$p" ] && continue

        src="$rel_root/$p"
        dst="$shared_root/$p"

        if [ ! -f "$src" ]; then
            echo "[shared] skip, src not found: $src"
            skipped=$((skipped + 1))
            continue
        fi

        mkdir -p "$(dirname "$dst")"

        if [ -f "$dst" ]; then
            rm -f "$src"
            ln "$dst" "$src"
            linked=$((linked + 1))
        else
            mv "$src" "$dst"
            ln "$dst" "$src"
            moved=$((moved + 1))
        fi
    done < "$hashes"

    echo "[shared] done: moved=$moved linked=$linked skipped=$skipped"
else
    echo "[shared] hashes file not found: $hashes"
fi

true"#,
        d = remote_deploy_path,
        t = timestamp,
        h = remote_hashes
    ));
}
    // 6) 切 current 到新版本
    commands.push(format!(
        "cd {} && ln -sfn releases/{} current",
        remote_deploy_path, timestamp
    ));

    // 7) 清理旧 release：只保留 keep_releases 个最新版本；删除时仅移除 shared 中“不再被任何保留的 release 引用”的文件
    if enable_shared {
        commands.push(format!(
            r#"set -e;
deploy_root="{d}";
releases_root="$deploy_root/releases";
shared_root="$deploy_root/shared";
keep="{k}";
tmp_root="{tmp}";
now_ts="{t}";

if [ -d "$releases_root" ]; then
    # 按时间倒序，保留前 keep 个，其余为待删除的 old_releases
    old_releases=$(cd "$releases_root" && ls -t | tail -n +$((keep + 1)) || true)

    for rel_name in $old_releases; do
        rel="$releases_root/$rel_name"
        [ -d "$rel" ] || continue

        manifest="$rel/shipfe.hashed_assets.txt"
        snapshot="$rel/shipfe.snapshot.json"
        refs_tmp="$tmp_root/shipfe_release_refs_${{now_ts}}_${{rel_name}}.txt"

        rm -f "$refs_tmp"
        touch "$refs_tmp"

        # 读取当前将被删除的 release 所引用的 hashed assets
        if [ -f "$manifest" ]; then
            cat "$manifest" >> "$refs_tmp"
        elif [ -f "$snapshot" ]; then
            awk '/"hashed_assets"[[:space:]]*:/ {{ in_list=1; next }}
                 in_list && /]/ {{ in_list=0; next }}
                 in_list {{
                     gsub(/^[[:space:]]*"/, "");
                     gsub(/",?[[:space:]]*$/, "");
                     if (length($0)) print
                 }}' "$snapshot" >> "$refs_tmp"
        fi

        sort -u "$refs_tmp" -o "$refs_tmp"

        # 删除 shared 中“仅被当前 release 使用”的文件
        if [ -d "$shared_root" ] && [ -s "$refs_tmp" ]; then
            while IFS= read -r rel_path; do
                [ -z "$rel_path" ] && continue

                still_used=0

                for other_rel in "$releases_root"/*; do
                    [ -d "$other_rel" ] || continue
                    [ "$other_rel" = "$rel" ] && continue

                    other_manifest="$other_rel/shipfe.hashed_assets.txt"
                    other_snapshot="$other_rel/shipfe.snapshot.json"

                    if [ -f "$other_manifest" ] && grep -Fqx -- "$rel_path" "$other_manifest"; then
                        still_used=1
                        break
                    fi

                    if [ -f "$other_snapshot" ] && awk -v target="$rel_path" '
                        BEGIN {{ found=0 }}
                        /"hashed_assets"[[:space:]]*:/ {{ in_list=1; next }}
                        in_list && /]/ {{ in_list=0; next }}
                        in_list {{
                            gsub(/^[[:space:]]*"/, "");
                            gsub(/",?[[:space:]]*$/, "");
                            if ($0 == target) {{
                                found=1;
                                exit 0
                            }}
                        }}
                        END {{
                            if (found == 1) exit 0;
                            exit 1;
                        }}
                    ' "$other_snapshot"; then
                        still_used=1
                        break
                    fi
                done

                if [ "$still_used" -eq 0 ]; then
                    shared_file="$shared_root/$rel_path"
                    [ -f "$shared_file" ] && rm -f "$shared_file"
                fi
            done < "$refs_tmp"

            find "$shared_root" -depth -type d -empty -delete || true
        fi

        rm -f "$refs_tmp"
        rm -rf "$rel"
    done
fi

true"#,
            d = remote_deploy_path,
            k = keep_releases,
            tmp = remote_tmp,
            t = timestamp,
        ));
    } else {
        // 不启用 shared 时，按原逻辑直接删旧 release
        commands.push(format!(
            "cd {} && ls -t releases/ | tail -n +{} | xargs -r -I {{}} rm -rf releases/{{}}",
            remote_deploy_path,
            keep_releases + 1
        ));
    }

    // 8) 删除远端临时文件
    commands.push(format!("rm -f {}", remote_archive));
    if enable_shared && !hashed_assets.is_empty() {
        commands.push(format!("rm -f {}", remote_hashes));
    }

    // 9) 逐条执行远端命令
    for cmd in commands {
        let mut channel = sess
            .channel_session()
            .map_err(|e| crate::AppError::Invalid(e.to_string()))?;

        channel
            .exec(&cmd)
            .map_err(|e| crate::AppError::Invalid(e.to_string()))?;

        channel
            .handle_extended_data(ssh2::ExtendedData::Merge)
            .map_err(|e| crate::AppError::Invalid(e.to_string()))?;

        let mut output = String::new();
        channel
            .read_to_string(&mut output)
            .map_err(|e| crate::AppError::Invalid(e.to_string()))?;

        channel
            .wait_close()
            .map_err(|e| crate::AppError::Invalid(e.to_string()))?;

        let status = channel
            .exit_status()
            .map_err(|e| crate::AppError::Invalid(e.to_string()))?;

        if status != 0 {
            return Err(crate::AppError::Invalid(format!(
                "Command failed: {}\n---- remote output ----\n{}",
                cmd, output
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