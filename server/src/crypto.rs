use aes_gcm::{aead::{Aead, KeyInit}, Aes256Gcm};
use argon2::{
    password_hash::{PasswordHash, PasswordHasher, PasswordVerifier, SaltString},
    Argon2,
};
use rand_core::{OsRng, RngCore};
use anyhow::Result;
use base64::{engine::general_purpose::STANDARD as BASE64, Engine};

const NONCE_LEN: usize = 12;

pub fn hash_password(password: &str) -> Result<String> {
    let salt = SaltString::generate(&mut OsRng);
    let argon2 = Argon2::default();
    let hash = argon2
        .hash_password(password.as_bytes(), &salt)
        .map_err(|e| anyhow::anyhow!("password hash failed: {}", e))?
        .to_string();
    Ok(hash)
}

pub fn verify_password(password: &str, hash: &str) -> Result<bool> {
    let parsed = PasswordHash::new(hash).map_err(|e| anyhow::anyhow!("invalid hash: {}", e))?;
    Ok(Argon2::default()
        .verify_password(password.as_bytes(), &parsed)
        .is_ok())
}

pub fn encrypt_text(plaintext: &str, key_b64: &str) -> Result<String> {
    let key_bytes = BASE64.decode(key_b64.trim()).map_err(|e| anyhow::anyhow!("invalid key: {}", e))?;
    if key_bytes.len() != 32 {
        anyhow::bail!("ENCRYPTION_KEY must be 32 bytes (base64)");
    }
    let cipher = Aes256Gcm::new_from_slice(&key_bytes).map_err(|e| anyhow::anyhow!("cipher init: {}", e))?;
    let mut nonce = [0u8; NONCE_LEN];
    OsRng.fill_bytes(&mut nonce);
    let ciphertext = cipher
        .encrypt((&nonce).into(), plaintext.as_bytes())
        .map_err(|e| anyhow::anyhow!("encrypt: {}", e))?;
    let mut out = Vec::with_capacity(NONCE_LEN + ciphertext.len());
    out.extend_from_slice(&nonce);
    out.extend_from_slice(&ciphertext);
    Ok(BASE64.encode(&out))
}

pub fn decrypt_text(encrypted_b64: &str, key_b64: &str) -> Result<String> {
    let key_bytes = BASE64.decode(key_b64.trim()).map_err(|e| anyhow::anyhow!("invalid key: {}", e))?;
    if key_bytes.len() != 32 {
        anyhow::bail!("ENCRYPTION_KEY must be 32 bytes (base64)");
    }
    let cipher = Aes256Gcm::new_from_slice(&key_bytes).map_err(|e| anyhow::anyhow!("cipher init: {}", e))?;
    let raw = BASE64.decode(encrypted_b64.trim()).map_err(|e| anyhow::anyhow!("invalid payload: {}", e))?;
    if raw.len() < NONCE_LEN {
        anyhow::bail!("payload too short");
    }
    let (nonce_slice, ciphertext) = raw.split_at(NONCE_LEN);
    let nonce = aes_gcm::Nonce::from_slice(nonce_slice);
    let plaintext = cipher
        .decrypt(nonce, ciphertext)
        .map_err(|_| anyhow::anyhow!("decrypt failed"))?;
    String::from_utf8(plaintext).map_err(|e| anyhow::anyhow!("utf8: {}", e))
}
