use base64::engine::general_purpose::STANDARD as b64;
use base64::Engine;
use ed25519_dalek::{SigningKey, Signer};
use serde::{Deserialize, Serialize};

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
    pub expires_at: Option<i64>,
    pub entitlements: Vec<Capability>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct LicenseFile {
    pub payload: LicensePayload,
    pub signature_b64: String,
}

fn main() {
    // 1) 你的私钥：真实场景应从安全存储读取。这里为了演示写死/环境变量都行
    // 你可以先运行一次生成固定私钥，并保存到文件
    let sk = load_or_create_signing_key();

    let machine_id = std::env::args().nth(1).expect("usage: issuer <machine_id>");
    let payload = LicensePayload {
        plan: Plan::Pro,
        machine_id,
        expires_at: None,
        entitlements: vec![Capability::AtomicSwitch, Capability::Rollback],
    };

    let msg = serde_json::to_vec(&payload).unwrap();
    let sig = sk.sign(&msg);

    let lf = LicenseFile {
        payload,
        signature_b64: b64.encode(sig.to_bytes()),
    };

    let out = serde_json::to_string_pretty(&lf).unwrap();
    println!("{out}");

    // 打印公钥，复制到客户端 verifying_key()
    let pk = sk.verifying_key();
    println!("PUBLIC_KEY_B64={}", b64.encode(pk.to_bytes()));
}

fn load_or_create_signing_key() -> SigningKey {
    // demo：每次生成都会变；真实要持久化私钥，否则以前签的 license 会全失效
    // 你第一次运行，把私钥 bytes 保存到文件，然后以后固定读取
    use rand_core::OsRng;
    SigningKey::generate(&mut OsRng)
}