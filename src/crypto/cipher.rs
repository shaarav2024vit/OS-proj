use std::fs::{self, File};
use std::io::{Read, Write};
use std::path::{Path, PathBuf};

use aes_gcm::aead::{Aead, KeyInit};
use aes_gcm::{Aes256Gcm, Nonce};
use argon2::{Algorithm, Argon2, Params, Version};
use rand::RngCore;
use zeroize::{Zeroize, ZeroizeOnDrop};

const MAGIC_BYTES: &[u8; 6] = b"STRATA";
const FORMAT_VERSION: u8 = 1;
const SALT_LEN: usize = 16;
const NONCE_LEN: usize = 12;

#[derive(Zeroize, ZeroizeOnDrop)]
struct DerivedKey([u8; 32]);

pub struct CryptoEngine;

impl CryptoEngine {
    /// Derive a 256-bit AES key from a passphrase using Argon2id
    fn derive_key(passphrase: &str, salt: &[u8]) -> Result<DerivedKey, String> {
        let mut key = [0u8; 32];
        let params = Params::new(19456, 2, 1, Some(32))
            .map_err(|e| format!("Argon2 params error: {}", e))?;
        let argon2 = Argon2::new(Algorithm::Argon2id, Version::V0x13, params);

        argon2
            .hash_password_into(passphrase.as_bytes(), salt, &mut key)
            .map_err(|e| format!("Key derivation error: {}", e))?;

        Ok(DerivedKey(key))
    }

    /// Encrypt a single file or a folder into a `.strata` file
    pub fn encrypt_path(path: &Path, passphrase: &str) -> Result<PathBuf, String> {
        if !path.exists() {
            return Err("Target path does not exist".into());
        }

        let is_dir = path.is_dir();
        let mut raw_bytes = Vec::new();

        if is_dir {
            // Bundle directory into a TAR byte buffer
            let mut tar_builder = tar::Builder::new(Vec::new());
            let dir_name = path
                .file_name()
                .and_then(|n| n.to_str())
                .unwrap_or("folder");

            tar_builder
                .append_dir_all(dir_name, path)
                .map_err(|e| format!("Failed to archive directory: {}", e))?;

            raw_bytes = tar_builder
                .into_inner()
                .map_err(|e| format!("Failed to finalize archive: {}", e))?;
        } else {
            let mut file = File::open(path).map_err(|e| format!("Failed to read file: {}", e))?;
            file.read_to_end(&mut raw_bytes)
                .map_err(|e| format!("Failed to read file data: {}", e))?;
        }

        // Generate cryptographically secure salt and nonce
        let mut salt = [0u8; SALT_LEN];
        let mut nonce_bytes = [0u8; NONCE_LEN];
        rand::thread_rng().fill_bytes(&mut salt);
        rand::thread_rng().fill_bytes(&mut nonce_bytes);

        let derived_key = Self::derive_key(passphrase, &salt)?;
        let cipher = Aes256Gcm::new_from_slice(&derived_key.0)
            .map_err(|e| format!("Cipher init error: {}", e))?;
        let nonce = Nonce::from_slice(&nonce_bytes);

        // Encrypt with AES-256-GCM
        let ciphertext = cipher
            .encrypt(nonce, raw_bytes.as_ref())
            .map_err(|e| format!("Encryption error: {}", e))?;

        // Determine destination file
        let output_path = if is_dir {
            path.with_extension("tar.strata")
        } else {
            let mut name = path.file_name().unwrap_or_default().to_os_string();
            name.push(".strata");
            path.with_file_name(name)
        };

        let temp_path = output_path.with_extension("strata.tmp");

        // Write atomic file envelope: MAGIC + VERSION + IS_DIR + SALT + NONCE + CIPHERTEXT
        let mut out =
            File::create(&temp_path).map_err(|e| format!("Failed to create file: {}", e))?;
        out.write_all(MAGIC_BYTES).map_err(|e| e.to_string())?;
        out.write_all(&[FORMAT_VERSION])
            .map_err(|e| e.to_string())?;
        out.write_all(&[if is_dir { 1 } else { 0 }])
            .map_err(|e| e.to_string())?;
        out.write_all(&salt).map_err(|e| e.to_string())?;
        out.write_all(&nonce_bytes).map_err(|e| e.to_string())?;
        out.write_all(&ciphertext).map_err(|e| e.to_string())?;
        out.sync_all().map_err(|e| e.to_string())?;

        // Atomically rename temporary file
        fs::rename(&temp_path, &output_path)
            .map_err(|e| format!("Failed to commit file: {}", e))?;

        // Safely remove unencrypted original
        if is_dir {
            let _ = fs::remove_dir_all(path);
        } else {
            let _ = fs::remove_file(path);
        }

        Ok(output_path)
    }

    /// Decrypt a `.strata` file back into its original file or directory
    pub fn decrypt_path(path: &Path, passphrase: &str) -> Result<PathBuf, String> {
        let mut file =
            File::open(path).map_err(|e| format!("Failed to open encrypted file: {}", e))?;
        let mut buffer = Vec::new();
        file.read_to_end(&mut buffer).map_err(|e| e.to_string())?;

        let header_len = 6 + 1 + 1 + SALT_LEN + NONCE_LEN;
        if buffer.len() < header_len {
            return Err("Invalid or corrupted Strata encrypted file".into());
        }

        if &buffer[0..6] != MAGIC_BYTES {
            return Err("Not a valid Strata file (invalid magic bytes)".into());
        }

        let is_dir = buffer[7] == 1;
        let salt = &buffer[8..8 + SALT_LEN];
        let nonce_bytes = &buffer[8 + SALT_LEN..header_len];
        let ciphertext = &buffer[header_len..];

        let derived_key = Self::derive_key(passphrase, salt)?;
        let cipher = Aes256Gcm::new_from_slice(&derived_key.0)
            .map_err(|e| format!("Cipher init error: {}", e))?;
        let nonce = Nonce::from_slice(nonce_bytes);

        let plaintext = cipher
            .decrypt(nonce, ciphertext)
            .map_err(|_| "Decryption failed! Incorrect password or corrupted file.".to_string())?;

        let parent = path.parent().unwrap_or_else(|| Path::new("."));

        if is_dir {
            // Unpack TAR
            let mut archive = tar::Archive::new(&plaintext[..]);
            archive
                .unpack(parent)
                .map_err(|e| format!("Failed to unpack folder: {}", e))?;

            // Remove .strata file
            let _ = fs::remove_file(path);

            let restored_name = path
                .file_name()
                .and_then(|n| n.to_str())
                .unwrap_or("")
                .trim_end_matches(".tar.strata")
                .trim_end_matches(".strata");
            Ok(parent.join(restored_name))
        } else {
            let original_name = path
                .file_name()
                .and_then(|n| n.to_str())
                .unwrap_or("")
                .trim_end_matches(".strata");

            let out_path = parent.join(original_name);
            let mut out = File::create(&out_path).map_err(|e| e.to_string())?;
            out.write_all(&plaintext).map_err(|e| e.to_string())?;
            out.sync_all().map_err(|e| e.to_string())?;

            let _ = fs::remove_file(path);
            Ok(out_path)
        }
    }
}
