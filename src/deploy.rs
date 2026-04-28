use chrono::Utc;
use flate2::write::GzEncoder;
use flate2::Compression;
use ssh2::Session;
use std::fs::{metadata, File, OpenOptions};
use std::io::{self, Read, Write};
use std::net::TcpStream;
use std::path::Path;
use std::time::{Duration, Instant};
use tar::Builder;

use crate::config::ServerConfig;

fn compress_dist(dist_path: &str, output_path: &str) -> Result<(), crate::AppError> {
    let file = File::create(output_path).map_err(|e| crate::AppError::Invalid(e.to_string()))?;
    let enc = GzEncoder::new(file, Compression::default());
    let mut tar = Builder::new(enc);

    tar.append_dir_all("dist", dist_path)
        .map_err(|e| crate::AppError::Invalid(e.to_string()))?;
    tar.finish()
        .map_err(|e| crate::AppError::Invalid(e.to_string()))?;

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
    print!("{}", log_entry);
    if let Ok(mut file) = OpenOptions::new()
        .create(true)
        .append(true)
        .open("shipfe.log")
    {
        let _ = file.write_all(log_entry.as_bytes());
    }
}

fn log_plain(message: &str) {
    if message.is_empty() {
        println!();
        if let Ok(mut file) = OpenOptions::new()
            .create(true)
            .append(true)
            .open("shipfe.log")
        {
            let _ = file.write_all(b"\n");
        }
        return;
    }

    let timestamp = Utc::now().format("%Y-%m-%d %H:%M:%S");
    let log_entry = format!("[{}] {}\n", timestamp, message);
    print!("{}", log_entry);
    if let Ok(mut file) = OpenOptions::new()
        .create(true)
        .append(true)
        .open("shipfe.log")
    {
        let _ = file.write_all(log_entry.as_bytes());
    }
}

fn format_elapsed(start: &Instant) -> String {
    let elapsed = start.elapsed();
    if elapsed.as_secs() > 0 {
        format!("{:.2}s", elapsed.as_secs_f64())
    } else {
        format!("{}ms", elapsed.as_millis())
    }
}

fn format_bytes(bytes: u64) -> String {
    const KIB: f64 = 1024.0;
    const MIB: f64 = KIB * 1024.0;

    if bytes as f64 >= MIB {
        format!("{:.2} MiB", bytes as f64 / MIB)
    } else if bytes as f64 >= KIB {
        format!("{:.2} KiB", bytes as f64 / KIB)
    } else {
        format!("{} B", bytes)
    }
}

fn copy_with_progress<R: Read, W: Write>(
    reader: &mut R,
    writer: &mut W,
    total_size: u64,
) -> Result<u64, crate::AppError> {
    let mut buffer = [0_u8; 64 * 1024];
    let mut copied = 0_u64;
    let mut last_log = Instant::now();

    if total_size > 0 {
        log_plain(&format!(
            "upload progress: 0 B / {} (0.0%)",
            format_bytes(total_size)
        ));
    }

    loop {
        let read = reader
            .read(&mut buffer)
            .map_err(|e| crate::AppError::Invalid(e.to_string()))?;
        if read == 0 {
            break;
        }

        writer
            .write_all(&buffer[..read])
            .map_err(|e| crate::AppError::Invalid(e.to_string()))?;
        copied += read as u64;

        if total_size > 0 && last_log.elapsed() >= Duration::from_secs(1) {
            let percent = copied as f64 * 100.0 / total_size as f64;
            log_plain(&format!(
                "upload progress: {} / {} ({:.1}%)",
                format_bytes(copied),
                format_bytes(total_size),
                percent.min(100.0)
            ));
            last_log = Instant::now();
        }
    }

    Ok(copied)
}

fn generate_snapshot(
    dist_path: &str,
    id: &str,
    patterns: &Option<Vec<String>>,
) -> Result<(), crate::AppError> {
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
                    let rel_path = path
                        .strip_prefix(base)
                        .unwrap()
                        .to_string_lossy()
                        .to_string();
                    files.push(rel_path.clone());

                    // ✅ 优先按用户 patterns（glob）识别 hashed assets
                    if let Some(pats) = patterns {
                        for pat in pats {
                            if glob::Pattern::new(pat)
                                .map_or(false, |pattern| pattern.matches(&rel_path))
                            {
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
                                    if after_dash.len() >= 6
                                        && after_dash.chars().all(|c| c.is_alphanumeric())
                                    {
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
    let json = serde_json::to_string_pretty(&snapshot)
        .map_err(|e| crate::AppError::Invalid(e.to_string()))?;
    std::fs::write(&snapshot_path, json).map_err(|e| crate::AppError::Invalid(e.to_string()))?;

    Ok(())
}

fn run_build_command(cmd: &str) -> Result<(), crate::AppError> {
    log_message(&format!("Running build command: {}", cmd));
    let build_start = Instant::now();
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
        log_message(&format!(
            "Build completed successfully in {}",
            format_elapsed(&build_start)
        ));
        Ok(())
    } else {
        log_message(&format!("Build failed in {}", format_elapsed(&build_start)));
        if !output.stderr.is_empty() {
            log_message(&format!(
                "Build error: {}",
                String::from_utf8_lossy(&output.stderr)
            ));
        }
        if output.stderr.is_empty() && !output.stdout.is_empty() {
            log_message(
                "Build stdout suppressed; rerun the build command manually for full output",
            );
        }
        Err(crate::AppError::Invalid(format!(
            "Build command failed: {}",
            cmd
        )))
    }
}

pub fn deploy_free(config: &crate::config::DeployParams) -> Result<(), crate::AppError> {
    log_plain("");
    log_plain("====== START =====");

    if let Some(build_cmd) = &config.build_command {
        run_build_command(build_cmd)?;
    }

    // 使用当前时间作为部署版本号，避免依赖 dist 目录 mtime 导致时间戳滞后
    let timestamp = Utc::now().format("%Y%m%d_%H%M%S").to_string();

    generate_snapshot(
        &config.local_dist_path,
        &timestamp,
        &config.hashed_asset_patterns,
    )?;

    let local_snapshot_path = format!("{}/shipfe.snapshot.json", config.local_dist_path);
    let hashed_assets: Vec<String> = if config.enable_shared {
        if let Ok(content) = std::fs::read_to_string(&local_snapshot_path) {
            serde_json::from_str::<Snapshot>(&content)
                .map(|s| s.hashed_assets)
                .unwrap_or_default()
        } else {
            Vec::new()
        }
    } else {
        Vec::new()
    };

    let archive_path = format!(
        "/tmp/shipfe_dist_{}_{}.tar.gz",
        timestamp,
        std::process::id()
    );
    compress_dist(&config.local_dist_path, &archive_path)?;

    let deploy_result: Result<(), crate::AppError> = (|| {
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
        Ok(())
    })();

    if let Err(err) = std::fs::remove_file(&archive_path) {
        log_message(&format!(
            "Warning: Failed to remove local archive {}: {}",
            archive_path, err
        ));
    }

    deploy_result?;

    log_plain("====== END =====");
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
    let server_total_start = Instant::now();
    let deploy_path = format!("{}/releases", remote_deploy_path);

    let tcp_start = Instant::now();
    let tcp = TcpStream::connect(format!("{}:{}", server.host, server.port))
        .map_err(|e| crate::AppError::Invalid(e.to_string()))?;
    log_plain(&format!("TCP: {}", format_elapsed(&tcp_start)));

    let mut sess = Session::new().map_err(|e| crate::AppError::Invalid(e.to_string()))?;
    sess.set_tcp_stream(tcp);
    let handshake_start = Instant::now();
    sess.handshake()
        .map_err(|e| crate::AppError::Invalid(e.to_string()))?;
    log_plain(&format!(
        "SSH handshake: {}",
        format_elapsed(&handshake_start)
    ));

    let auth_start = Instant::now();
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
    log_plain(&format!("SSH auth: {}", format_elapsed(&auth_start)));

    // 1) 上传 dist.tar.gz 到远端临时目录（使用唯一文件名避免并发部署互相覆盖）
    let pid = std::process::id();
    let remote_archive = format!("{}/shipfe_dist_{}_{}.tar.gz", remote_tmp, timestamp, pid);
    let file_size = metadata(local_archive)
        .map_err(|e| crate::AppError::Invalid(e.to_string()))?
        .len();

    let archive_upload_start = Instant::now();
    let mut remote_file = sess
        .scp_send(Path::new(&remote_archive), 0o644, file_size, None)
        .map_err(|e| crate::AppError::Invalid(e.to_string()))?;

    let mut local_file =
        File::open(local_archive).map_err(|e| crate::AppError::Invalid(e.to_string()))?;
    copy_with_progress(&mut local_file, &mut remote_file, file_size)?;
    log_plain(&format!(
        "upload {}: {}",
        format_bytes(file_size),
        format_elapsed(&archive_upload_start)
    ));

    // 2) 如果启用 shared，则上传当前发布的 hash 资源清单（唯一文件名）
    let remote_hashes = format!(
        "{}/shipfe_current_hashes_{}_{}.txt",
        remote_tmp, timestamp, pid
    );

    if enable_shared && !hashed_assets.is_empty() {
        let local_hashes_path = format!("/tmp/shipfe_current_hashes_{}_{}.txt", timestamp, pid);
        let hash_lines = hashed_assets.join("\n");

        std::fs::write(&local_hashes_path, hash_lines)
            .map_err(|e| crate::AppError::Invalid(e.to_string()))?;

        let hashes_size = metadata(&local_hashes_path)
            .map_err(|e| crate::AppError::Invalid(e.to_string()))?
            .len();

        let mut remote_hashes_file = sess
            .scp_send(Path::new(&remote_hashes), 0o644, hashes_size, None)
            .map_err(|e| crate::AppError::Invalid(e.to_string()))?;

        let mut local_hashes_file =
            File::open(&local_hashes_path).map_err(|e| crate::AppError::Invalid(e.to_string()))?;

        io::copy(&mut local_hashes_file, &mut remote_hashes_file)
            .map_err(|e| crate::AppError::Invalid(e.to_string()))?;

        if let Err(err) = std::fs::remove_file(&local_hashes_path) {
            log_message(&format!(
                "Warning: Failed to remove local hash manifest {}: {}",
                local_hashes_path, err
            ));
        }
    }

    let mut commands: Vec<(&str, String)> = vec![];

    // 3) 基础目录准备
    commands.push((
        "prepare releases directory",
        format!("mkdir -p {}", deploy_path),
    ));
    if enable_shared {
        commands.push((
            "prepare shared directory",
            format!("mkdir -p {}/shared", remote_deploy_path),
        ));
    }

    // 4) 创建本次 release 目录并解压
    commands.push((
        "create release directory",
        format!("cd {} && mkdir -p {}", deploy_path, timestamp),
    ));
    commands.push((
        "extract archive",
        format!(
            "cd {} && tar -xzf {} -C {} --strip-components=1",
            deploy_path, remote_archive, timestamp
        ),
    ));

    // 5) 如果启用 shared：按 hashed_asset_patterns 直接把匹配到的文件复制进 shared，
    //    保持目录结构；同名文件全部覆盖；release 自身保持完整，不做硬链接/搬移。
    if enable_shared && !hashed_assets.is_empty() {
        commands.push((
            "sync shared assets",
            format!(
                r#"set -e;
rel_root="{d}/releases/{t}";
shared_root="{d}/shared";
hashes="{h}";

mkdir -p "$shared_root";

if [ -f "$hashes" ]; then
    copied=0
    overwritten=0
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
            overwritten=$((overwritten + 1))
        fi

        cp -f "$src" "$dst"
        copied=$((copied + 1))
    done < "$hashes"

    echo "[shared] done: copied=$copied overwritten=$overwritten skipped=$skipped"
else
    echo "[shared] hashes file not found: $hashes"
fi

true"#,
                d = remote_deploy_path,
                t = timestamp,
                h = remote_hashes
            ),
        ));
    }
    // 6) 切 current 到新版本
    commands.push((
        "switch current symlink",
        format!(
            "cd {} && ln -sfn releases/{} current",
            remote_deploy_path, timestamp
        ),
    ));

    // 7) 清理旧 release：只按 keep_releases 触发；对被删 release，只依据它自己的 snapshot
    //    删 shared 里对应的那些文件；若文件仍被其他保留 release 的 snapshot 引用则保留。
    //    不会动"对应不上这个被删 release"的 shared 文件。
    if enable_shared {
        commands.push((
            "cleanup old releases and shared assets",
            format!(
                r#"set -e;
deploy_root="{d}";
releases_root="$deploy_root/releases";
shared_root="$deploy_root/shared";
keep="{k}";
tmp_root="{tmp}";
now_ts="{t}";
cleanup_deleted_release=0
cleanup_deleted_shared=0

mkdir -p "$tmp_root"

if [ "$keep" -eq 0 ]; then
    echo "[cleanup] keep_releases=0, skip release cleanup"
elif [ -d "$releases_root" ]; then
    # 仅统计 release 目录，按时间戳名称倒序，保留前 keep 个
    old_releases=$(
        find "$releases_root" -mindepth 1 -maxdepth 1 -type d 2>/dev/null \
            | sed "s#^$releases_root/##" \
            | grep -E '^[0-9]{{8}}_[0-9]{{6}}$' \
            | sort -r \
            | tail -n +$((keep + 1)) || true
    )

    for rel_name in $old_releases; do
        rel="$releases_root/$rel_name"
        [ -d "$rel" ] || continue

        snapshot="$rel/shipfe.snapshot.json"
        refs_tmp="$tmp_root/shipfe_release_refs_${{now_ts}}_${{rel_name}}.txt"

        rm -f "$refs_tmp"
        touch "$refs_tmp"

        # 读取将被删除 release 自己 snapshot 里登记过的 shared 文件路径
        if [ -f "$snapshot" ]; then
            awk '/"hashed_assets"[[:space:]]*:/ {{ in_list=1; next }}
                 in_list && /]/ {{ in_list=0; next }}
                 in_list {{
                     gsub(/^[[:space:]]*"/, "");
                     gsub(/",?[[:space:]]*$/, "");
                     if (length($0)) print
                 }}' "$snapshot" >> "$refs_tmp"
        fi

        sort -u "$refs_tmp" -o "$refs_tmp"

        # 仅在 shared 中删除"被删 release 登记 且 其他保留 release 都不再引用"的文件
        if [ -d "$shared_root" ] && [ -s "$refs_tmp" ]; then
            while IFS= read -r rel_path; do
                [ -z "$rel_path" ] && continue

                still_used=0
                for other_rel in "$releases_root"/*; do
                    [ -d "$other_rel" ] || continue
                    [ "$other_rel" = "$rel" ] && continue

                    other_snapshot="$other_rel/shipfe.snapshot.json"

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
                    if [ -f "$shared_file" ]; then
                        rm -f "$shared_file"
                        cleanup_deleted_shared=$((cleanup_deleted_shared + 1))
                    fi
                fi
            done < "$refs_tmp"

            find "$shared_root" -depth -type d -empty -delete || true
        fi

        rm -f "$refs_tmp"
        rm -rf "$rel"
        cleanup_deleted_release=$((cleanup_deleted_release + 1))
    done
fi

kept_release_count=0
shared_total_count=0
if [ -d "$releases_root" ]; then
    kept_release_count=$(
        find "$releases_root" -mindepth 1 -maxdepth 1 -type d 2>/dev/null \
            | sed "s#^$releases_root/##" \
            | grep -E '^[0-9]{{8}}_[0-9]{{6}}$' \
            | wc -l | tr -d ' '
    )
fi
if [ -d "$shared_root" ]; then
    shared_total_count=$(find "$shared_root" -type f 2>/dev/null | wc -l | tr -d ' ')
fi
echo "[cleanup] done: releases_deleted=$cleanup_deleted_release shared_deleted=$cleanup_deleted_shared releases_kept=$kept_release_count shared_files=$shared_total_count"

true"#,
            d = remote_deploy_path,
            k = keep_releases,
            tmp = remote_tmp,
            t = timestamp,
            ),
        ));
    } else {
        // 不启用 shared 时，按同样规则直接删旧 release
        commands.push((
            "cleanup old releases",
            format!(
                r#"set -e;
deploy_root="{d}";
releases_root="$deploy_root/releases";
keep="{k}";

if [ "$keep" -eq 0 ]; then
    echo "[cleanup] keep_releases=0, skip release cleanup"
elif [ -d "$releases_root" ]; then
    old_releases=$(
        find "$releases_root" -mindepth 1 -maxdepth 1 -type d 2>/dev/null \
            | sed "s#^$releases_root/##" \
            | grep -E '^[0-9]{{8}}_[0-9]{{6}}$' \
            | sort -r \
            | tail -n +$((keep + 1)) || true
    )
    for rel_name in $old_releases; do
        rm -rf "$releases_root/$rel_name"
    done
fi

true"#,
                d = remote_deploy_path,
                k = keep_releases,
            ),
        ));
    }

    // 8) 删除远端临时文件
    commands.push(("remove remote archive", format!("rm -f {}", remote_archive)));
    if enable_shared && !hashed_assets.is_empty() {
        commands.push((
            "remove shared asset manifest",
            format!("rm -f {}", remote_hashes),
        ));
    }

    // 9) 逐条执行远端命令
    for (label, cmd) in commands {
        let remote_step_start = Instant::now();
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

        match label {
            "extract archive" => {
                log_plain(&format!("unzip: {}", format_elapsed(&remote_step_start)));
            }
            "sync shared assets" => {
                log_plain(&format!(
                    "sync shared assets: {}",
                    format_elapsed(&remote_step_start)
                ));
            }
            "cleanup old releases and shared assets" | "cleanup old releases" => {
                log_plain(&format!("cleanup: {}", format_elapsed(&remote_step_start)));
            }
            _ => {}
        }
    }

    log_plain(&format!(
        "Server deployment total: {}",
        format_elapsed(&server_total_start)
    ));

    Ok(())
}

pub fn rollback_to_version(
    server: &ServerConfig,
    remote_deploy_path: &str,
    to_version: &str,
) -> Result<(), crate::AppError> {
    log_message(&format!(
        "Connecting to server {}:{}",
        server.host, server.port
    ));
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

    // Check if the target version exists
    let check_cmd = format!("test -d {}/releases/{}", remote_deploy_path, to_version);
    let mut channel = sess
        .channel_session()
        .map_err(|e| crate::AppError::Invalid(e.to_string()))?;
    channel
        .exec(&check_cmd)
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
            "Version {} does not exist on server",
            to_version
        )));
    }

    // Update the current symlink
    let rollback_cmd = format!(
        "cd {} && ln -sfn releases/{} current",
        remote_deploy_path, to_version
    );
    let mut channel = sess
        .channel_session()
        .map_err(|e| crate::AppError::Invalid(e.to_string()))?;
    channel
        .exec(&rollback_cmd)
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
            "Failed to rollback to version {}: {}",
            to_version, output
        )));
    }

    log_message(&format!(
        "Successfully rolled back to version {}",
        to_version
    ));
    Ok(())
}
