use serde::{Deserialize, Serialize};
use std::{collections::HashSet, fs, path::PathBuf};

use base64::Engine;
use base64::engine::general_purpose::STANDARD as b64;
use ed25519_dalek::{Signature, Verifier, VerifyingKey};
use sha2::{Digest, Sha256};
use thiserror::Error;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Capability {
    AtomicSwitch,
    Rollback,
    MultiServer,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum Plan {
    Free,
    Pro,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct LicensePayload {
    pub plan: Plan,
    pub machine_id: String,
    pub expires_at: Option<i64>, // unix timestamp
    pub entitlements: Vec<Capability>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct LicenseFile {
    pub payload: LicensePayload,
    pub signature_b64: String,
}

#[derive(Debug, Error)]
pub enum LicenseError {
    #[error("license not found")]
    NotFound,
    #[error("license invalid: {0}")]
    Invalid(String),
    #[error("missing capability: {0:?}")]
    MissingCapability(Capability),
}

// ========== 目录结构：config_dir()/shipfe/profiles/<profile>/... ==========

pub fn app_dir() -> Result<PathBuf, LicenseError> {
    let base = dirs::config_dir()
        .ok_or_else(|| LicenseError::Invalid("no config dir".into()))?;
    Ok(base.join("shipfe"))
}

pub fn profiles_dir() -> Result<PathBuf, LicenseError> {
    Ok(app_dir()?.join("profiles"))
}

pub fn profile_dir(profile: &str) -> Result<PathBuf, LicenseError> {
    Ok(profiles_dir()?.join(profile))
}

pub fn ensure_profile_dir(profile: &str) -> Result<(), LicenseError> {
    let dir = profile_dir(profile)?;
    fs::create_dir_all(&dir).map_err(|e| LicenseError::Invalid(e.to_string()))?;
    Ok(())
}

pub fn license_path(profile: &str) -> Result<PathBuf, LicenseError> {
    Ok(profile_dir(profile)?.join("license.json"))
}

pub fn machine_id_path(profile: &str) -> Result<PathBuf, LicenseError> {
    Ok(profile_dir(profile)?.join("machine_id"))
}

// ========== machine_id：每个 profile 独立一个 ==========

pub fn get_or_create_machine_id(profile: &str) -> Result<String, LicenseError> {
    ensure_profile_dir(profile)?;
    let p = machine_id_path(profile)?;

    if let Ok(s) = fs::read_to_string(&p) {
        let s = s.trim().to_string();
        if !s.is_empty() {
            return Ok(s);
        }
    }

    // 简单指纹：hostname + username + os + 时间盐
    let host = gethostname::gethostname().to_string_lossy().to_string();
    let user = whoami::username();
    let os = whoami::platform().to_string();
    let salt = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_nanos()
        .to_string();

    let raw = format!("{}|{}|{}|{}", host, user, os, salt);
    let mut h = Sha256::new();
    h.update(raw.as_bytes());
    let id = hex::encode(h.finalize());

    fs::write(&p, &id).map_err(|e| LicenseError::Invalid(e.to_string()))?;
    Ok(id)
}

// ========== license 文件读写（按 profile） ==========

pub fn load_license(profile: &str) -> Result<LicenseFile, LicenseError> {
    let p = license_path(profile)?;
    let s = fs::read_to_string(&p).map_err(|_| LicenseError::NotFound)?;
    serde_json::from_str(&s).map_err(|e| LicenseError::Invalid(e.to_string()))
}

pub fn save_license_file(profile: &str, raw_json: &str) -> Result<(), LicenseError> {
    ensure_profile_dir(profile)?;
    let p = license_path(profile)?;
    fs::write(&p, raw_json).map_err(|e| LicenseError::Invalid(e.to_string()))?;
    Ok(())
}

// ========== 验签相关（你原逻辑保留/替换即可） ==========

pub fn verifying_key() -> Result<VerifyingKey, LicenseError> {
    Err(LicenseError::Invalid("verifying key not set yet".into()))
}

pub fn canonical_payload_json(payload: &LicensePayload) -> Result<Vec<u8>, LicenseError> {
    serde_json::to_vec(payload).map_err(|e| LicenseError::Invalid(e.to_string()))
}

pub fn verify_license(
    lf: &LicenseFile,
    expected_machine_id: &str,
) -> Result<(), LicenseError> {
    if lf.payload.machine_id != expected_machine_id {
        return Err(LicenseError::Invalid("machine_id mismatch".into()));
    }

    if let Some(exp) = lf.payload.expires_at {
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs() as i64;
        if now > exp {
            return Err(LicenseError::Invalid("license expired".into()));
        }
    }

    let vk = verifying_key()?;
    let msg = canonical_payload_json(&lf.payload)?;
    let sig_bytes = b64
        .decode(lf.signature_b64.as_bytes())
        .map_err(|e| LicenseError::Invalid(e.to_string()))?;
    let sig =
        Signature::from_slice(&sig_bytes).map_err(|e| LicenseError::Invalid(e.to_string()))?;

    vk.verify(&msg, &sig)
        .map_err(|e| LicenseError::Invalid(e.to_string()))?;

    Ok(())
}

// ========== LicenseCtx：按 profile 加载 ==========

pub enum LicenseCtx {
    Free,
    Pro { entitlements: HashSet<Capability> },
}

impl LicenseCtx {
    pub fn from_file_or_free(profile: &str) -> Result<Self, LicenseError> {
        let machine_id = get_or_create_machine_id(profile)?;

        let lf = match load_license(profile) {
            Ok(v) => v,
            Err(LicenseError::NotFound) => return Ok(LicenseCtx::Free),
            Err(e) => return Err(e),
        };

        verify_license(&lf, &machine_id)?;

        Ok(LicenseCtx::Pro {
            entitlements: lf.payload.entitlements.into_iter().collect(),
        })
    }

    pub fn require(&self, cap: Capability) -> Result<(), LicenseError> {
        match self {
            LicenseCtx::Free => Err(LicenseError::MissingCapability(cap)),
            LicenseCtx::Pro { entitlements } => {
                if entitlements.contains(&cap) {
                    Ok(())
                } else {
                    Err(LicenseError::MissingCapability(cap))
                }
            }
        }
    }
}