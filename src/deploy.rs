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
    log_message(&format!("正在执行构建命令: {}", cmd));
    use std::process::Command;
    let output = Command::new("sh")
        .arg("-c")
        .arg(cmd)
        .output()
        .map_err(|e| {
            log_message(&format!("构建命令执行失败: {}", e));
            LicenseError::Invalid(format!("Failed to run build command: {}", e))
        })?;
    
    if output.status.success() {
        log_message("构建成功");
        if !output.stdout.is_empty() {
            log_message(&format!("构建输出: {}", String::from_utf8_lossy(&output.stdout)));
        }
    } else {
        log_message("构建失败");
        if !output.stderr.is_empty() {
            log_message(&format!("构建错误: {}", String::from_utf8_lossy(&output.stderr)));
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
    log_message(&format!("正在压缩 {} 到 {}", config.local_dist_path, archive_path));
    compress_dist(&config.local_dist_path, archive_path)?;
    log_message("压缩完成");

    // Free版本只支持单服务器
    if config.servers.len() != 1 {
        return Err(LicenseError::Invalid("Free plan only supports single server".to_string()));
    }

    let server = &config.servers[0];
    upload_and_deploy(server, archive_path, &server.remote_deploy_path, &config.remote_tmp, server.delete_old)?;

    log_message("部署成功完成");
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
    log_message(&format!("正在连接到服务器 {}:{}", server.host, server.port));
    let tcp = TcpStream::connect(format!("{}:{}", server.host, server.port)).map_err(|e| {
        log_message(&format!("连接失败: {}", e));
        LicenseError::Invalid(e.to_string())
    })?;
    log_message("连接成功");
    let mut sess = Session::new().map_err(|e| LicenseError::Invalid(e.to_string()))?;
    sess.set_tcp_stream(tcp);
    log_message("正在进行SSH握手");
    sess.handshake().map_err(|e| {
        log_message(&format!("SSH握手失败: {}", e));
        LicenseError::Invalid(e.to_string())
    })?;
    log_message("SSH握手成功");

    // 认证
    log_message("正在进行SSH认证");
    let auth_success = if let Some(password) = &server.password {
        log_message("尝试密码认证");
        sess.userauth_password(&server.username, password).is_ok()
    } else if let Ok(private_key) = std::env::var("SSH_PRIVATE_KEY") {
        log_message("尝试SSH私钥认证");
        sess.userauth_pubkey_memory(&server.username, None, &private_key, None).is_ok()
    } else if let Some(key_path) = &server.key_path {
        log_message("尝试SSH密钥文件认证");
        sess.userauth_pubkey_file(&server.username, None, std::path::Path::new(key_path), None).is_ok()
    } else {
        false
    };

    if !auth_success {
        log_message("所有SSH认证方法都失败了");
        return Err(LicenseError::Invalid("SSH authentication failed".to_string()));
    }
    log_message("SSH认证成功");

    // 上传文件
    log_message("正在上传文件到服务器");
    let mut remote_file = sess.scp_send(std::path::Path::new(local_archive), 0o644, local_archive.len() as u64, None).map_err(|e| {
        log_message(&format!("文件上传初始化失败: {}", e));
        LicenseError::Invalid(e.to_string())
    })?;
    let mut local_file = File::open(local_archive).map_err(|e| {
        log_message(&format!("本地文件打开失败: {}", e));
        LicenseError::Invalid(e.to_string())
    })?;
    io::copy(&mut local_file, &mut remote_file).map_err(|e| {
        log_message(&format!("文件上传失败: {}", e));
        LicenseError::Invalid(e.to_string())
    })?;
    log_message("文件上传成功");

    // 执行部署命令
    let timestamp = Utc::now().format("%Y%m%d_%H%M%S");
    let remote_archive = format!("{}/dist.tar.gz", remote_tmp);

    let commands = vec![
        format!("mkdir -p {}", deploy_path),
        format!("mv {} {}", local_archive, remote_archive),
        format!("cd {} && mv dist dist_backup_{}", deploy_path, timestamp),
        format!("cd {} && tar -xzf {}", deploy_path, remote_archive),
        if delete_old {
            format!("cd {} && rm -rf dist_backup_{}", deploy_path, timestamp)
        } else {
            format!("cd {} && mv dist_backup_{} old_dist_{}", deploy_path, timestamp, timestamp)
        },
        format!("rm {}", remote_archive),
    ];

    println!("[{}] 开始执行部署命令", Utc::now().format("%Y-%m-%d %H:%M:%S"));
    for cmd in commands {
        println!("[{}] 执行命令: {}", Utc::now().format("%Y-%m-%d %H:%M:%S"), cmd);
        let mut channel = sess.channel_session().map_err(|e| {
            println!("[{}] 创建SSH通道失败: {}", Utc::now().format("%Y-%m-%d %H:%M:%S"), e);
            LicenseError::Invalid(e.to_string())
        })?;
        channel.exec(&cmd).map_err(|e| {
            println!("[{}] 命令执行失败: {}", Utc::now().format("%Y-%m-%d %H:%M:%S"), e);
            LicenseError::Invalid(e.to_string())
        })?;
        let mut output = String::new();
        channel.read_to_string(&mut output).map_err(|e| {
            println!("[{}] 读取命令输出失败: {}", Utc::now().format("%Y-%m-%d %H:%M:%S"), e);
            LicenseError::Invalid(e.to_string())
        })?;
        channel.wait_close().map_err(|e| {
            println!("[{}] 等待命令完成失败: {}", Utc::now().format("%Y-%m-%d %H:%M:%S"), e);
            LicenseError::Invalid(e.to_string())
        })?;
        if channel.exit_status().map_err(|e| {
            println!("[{}] 获取退出状态失败: {}", Utc::now().format("%Y-%m-%d %H:%M:%S"), e);
            LicenseError::Invalid(e.to_string())
        })? != 0 {
            println!("[{}] 命令执行失败: {}", Utc::now().format("%Y-%m-%d %H:%M:%S"), cmd);
            if !output.is_empty() {
                println!("命令输出: {}", output);
            }
            return Err(LicenseError::Invalid(format!("Command failed: {}", cmd)));
        } else {
            println!("[{}] 命令执行成功", Utc::now().format("%Y-%m-%d %H:%M:%S"));
            if !output.is_empty() {
                println!("命令输出: {}", output.trim());
            }
        }
    }
    println!("[{}] 部署完成", Utc::now().format("%Y-%m-%d %H:%M:%S"));

    Ok(())
}