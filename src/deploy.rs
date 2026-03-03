use std::fs::{File, OpenOptions};
use std::io::{self, Read, Write};
use tar::Builder;
use flate2::Compression;
use flate2::write::GzEncoder;
use ssh2::Session;
use std::net::TcpStream;
use chrono::Utc;


use crate::config::ServerConfig;
use crate::LicenseError;

fn log_message(message: &str) {
    let timestamp = Utc::now().format("%Y-%m-%d %H:%M:%S");
    let log_entry = format!("[{}] {}\n", timestamp, message);
    println!("{}", message); // 保留控制台输出
    if let Ok(mut file) = OpenOptions::new().create(true).append(true).open("shipfe.log") {
        let _ = file.write_all(log_entry.as_bytes());
    }
}

fn run_build_command(cmd: &str) -> Result<(), LicenseError> {
    log_message(&format!("Running build command: {}", cmd));
    use std::process::Command;
    let output = Command::new("sh")
        .arg("-c")
        .arg(cmd)
        .output()
        .map_err(|e| {
            log_message(&format!("Build command execution failed: {}", e));
            LicenseError::Invalid(format!("Failed to run build command: {}", e))
        })?;
    
    if output.status.success() {
        log_message("Build successful");
        if !output.stdout.is_empty() {
            log_message(&format!("Build output: {}", String::from_utf8_lossy(&output.stdout)));
        }
    } else {
        log_message("Build failed");
        if !output.stderr.is_empty() {
            log_message(&format!("Build error: {}", String::from_utf8_lossy(&output.stderr)));
        }
        return Err(LicenseError::Invalid(format!("Build command failed: {}", cmd)));
    }
    Ok(())
}

pub fn deploy_free(config: &crate::config::DeployParams) -> Result<(), LicenseError> {
    // 执行打包命令
    if let Some(build_cmd) = &config.build_command {
        run_build_command(build_cmd)?;
    }

    // 压缩dist目录
    let archive_path = "/tmp/dist.tar.gz";
    log_message(&format!("Compressing {} to {}", config.local_dist_path, archive_path));
    compress_dist(&config.local_dist_path, archive_path)?;
    log_message("Compression completed");

    // Free版本只支持单服务器
    if config.servers.len() != 1 {
        return Err(LicenseError::Invalid("Free plan only supports single server".to_string()));
    }

    let server = &config.servers[0];
    upload_and_deploy(server, archive_path, &server.remote_deploy_path, &config.remote_tmp, server.delete_old)?;

    log_message("Deployment completed successfully");
    Ok(())
}

fn compress_dist(dist_path: &str, output_path: &str) -> Result<(), LicenseError> {
    let file = File::create(output_path).map_err(|e| LicenseError::Invalid(e.to_string()))?;
    let enc = GzEncoder::new(file, Compression::default());
    let mut tar = Builder::new(enc);

    tar.append_dir_all("dist", dist_path).map_err(|e| LicenseError::Invalid(e.to_string()))?;
    tar.finish().map_err(|e| LicenseError::Invalid(e.to_string()))?;

    Ok(())
}

fn upload_and_deploy(server: &ServerConfig, local_archive: &str, deploy_path: &str, remote_tmp: &str, delete_old: bool) -> Result<(), LicenseError> {
    log_message(&format!("Connecting to server {}:{}", server.host, server.port));
    let tcp = TcpStream::connect(format!("{}:{}", server.host, server.port)).map_err(|e| {
        log_message(&format!("Connection failed: {}", e));
        LicenseError::Invalid(e.to_string())
    })?;
    log_message("Connection successful");
    let mut sess = Session::new().map_err(|e| LicenseError::Invalid(e.to_string()))?;
    sess.set_tcp_stream(tcp);
    log_message("Performing SSH handshake");
    sess.handshake().map_err(|e| {
        log_message(&format!("SSH handshake failed: {}", e));
        LicenseError::Invalid(e.to_string())
    })?;
    log_message("SSH handshake successful");

    // 认证
    log_message("Performing SSH authentication");
    let auth_success = if let Some(password) = &server.password {
        log_message("Attempting password authentication");
        sess.userauth_password(&server.username, password).is_ok()
    } else if let Ok(private_key) = std::env::var("SSH_PRIVATE_KEY") {
        log_message("Attempting SSH private key authentication");
        sess.userauth_pubkey_memory(&server.username, None, &private_key, None).is_ok()
    } else if let Some(key_path) = &server.key_path {
        log_message("Attempting SSH key file authentication");
        sess.userauth_pubkey_file(&server.username, None, std::path::Path::new(key_path), None).is_ok()
    } else {
        false
    };

    if !auth_success {
        log_message("All SSH authentication methods failed");
        return Err(LicenseError::Invalid("SSH authentication failed".to_string()));
    }
    log_message("SSH authentication successful");

    // 上传文件
    log_message("Uploading files to server");
    let remote_archive = format!("{}/dist.tar.gz", remote_tmp);
    let mut remote_file = sess.scp_send(std::path::Path::new(&remote_archive), 0o644, local_archive.len() as u64, None).map_err(|e| {
        log_message(&format!("File upload initialization failed: {}", e));
        LicenseError::Invalid(e.to_string())
    })?;
    let mut local_file = File::open(local_archive).map_err(|e| {
        log_message(&format!("Local file open failed: {}", e));
        LicenseError::Invalid(e.to_string())
    })?;
    io::copy(&mut local_file, &mut remote_file).map_err(|e| {
        log_message(&format!("File upload failed: {}", e));
        LicenseError::Invalid(e.to_string())
    })?;
    log_message("File upload successful");

    // 验证上传的文件是否存在
    let check_commands = vec![
        format!("ls -la {}", remote_archive),
        format!("test -f {} && echo 'File exists, size: $(stat -f%z {} 2>/dev/null || stat -c%s {} 2>/dev/null || echo \"unknown\")' || echo 'File does not exist'", remote_archive, remote_archive, remote_archive),
    ];

    log_message("Verifying uploaded file...");
    for cmd in check_commands {
        log_message(&format!("Running verification command: {}", cmd));
        let mut channel = sess.channel_session().map_err(|e| {
            log_message(&format!("SSH channel creation failed: {}", e));
            LicenseError::Invalid(e.to_string())
        })?;
        channel.exec(&cmd).map_err(|e| {
            log_message(&format!("Verification command failed: {}", e));
            LicenseError::Invalid(e.to_string())
        })?;
        let mut output = String::new();
        channel.read_to_string(&mut output).map_err(|e| {
            log_message(&format!("Reading verification output failed: {}", e));
            LicenseError::Invalid(e.to_string())
        })?;
        channel.wait_close().map_err(|e| {
            log_message(&format!("Waiting for verification completion failed: {}", e));
            LicenseError::Invalid(e.to_string())
        })?;
        if !output.is_empty() {
            log_message(&format!("Verification output: {}", output.trim()));
        }
    }

    // 执行部署命令
    let timestamp = Utc::now().format("%Y%m%d_%H%M%S");

    let commands = vec![
        format!("mkdir -p {}", deploy_path),
        format!("cd {} && [ -d dist ] && mv dist dist_backup_{} || true", deploy_path, timestamp),
        format!("cd {} && tar -xzf {}", deploy_path, remote_archive),
        if delete_old {
            format!("cd {} && [ -d dist_backup_{} ] && rm -rf dist_backup_{} || true", deploy_path, timestamp, timestamp)
        } else {
            format!("cd {} && [ -d dist_backup_{} ] && mv dist_backup_{} old_dist_{} || true", deploy_path, timestamp, timestamp, timestamp)
        },
        format!("rm {}", remote_archive),
    ];

    println!("[{}] Starting deployment commands", Utc::now().format("%Y-%m-%d %H:%M:%S"));
    for cmd in commands {
        println!("[{}] Executing command: {}", Utc::now().format("%Y-%m-%d %H:%M:%S"), cmd);
        let mut channel = sess.channel_session().map_err(|e| {
            println!("[{}] SSH channel creation failed: {}", Utc::now().format("%Y-%m-%d %H:%M:%S"), e);
            LicenseError::Invalid(e.to_string())
        })?;
        channel.exec(&cmd).map_err(|e| {
            println!("[{}] Command execution failed: {}", Utc::now().format("%Y-%m-%d %H:%M:%S"), e);
            LicenseError::Invalid(e.to_string())
        })?;
        let mut output = String::new();
        channel.read_to_string(&mut output).map_err(|e| {
            println!("[{}] Reading command output failed: {}", Utc::now().format("%Y-%m-%d %H:%M:%S"), e);
            LicenseError::Invalid(e.to_string())
        })?;
        channel.wait_close().map_err(|e| {
            println!("[{}] Waiting for command completion failed: {}", Utc::now().format("%Y-%m-%d %H:%M:%S"), e);
            LicenseError::Invalid(e.to_string())
        })?;
        if channel.exit_status().map_err(|e| {
            println!("[{}] Getting exit status failed: {}", Utc::now().format("%Y-%m-%d %H:%M:%S"), e);
            LicenseError::Invalid(e.to_string())
        })? != 0 {
            println!("[{}] Command execution failed: {}", Utc::now().format("%Y-%m-%d %H:%M:%S"), cmd);
            if !output.is_empty() {
                println!("Command output: {}", output);
            }
            return Err(LicenseError::Invalid(format!("Command failed: {}", cmd)));
        } else {
            println!("[{}] Command executed successfully", Utc::now().format("%Y-%m-%d %H:%M:%S"));
            if !output.is_empty() {
                println!("Command output: {}", output.trim());
            }
        }
    }
    println!("[{}] Deployment completed", Utc::now().format("%Y-%m-%d %H:%M:%S"));

    Ok(())
}