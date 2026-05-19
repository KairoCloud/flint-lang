use std::collections::HashMap;
use std::sync::{Arc, Mutex};

pub fn encrypt_env(key: &str, value: &str) -> Result<String, String> {
    if key.is_empty() {
        return Err("empty key".to_string());
    }
    
    // Simple XOR encryption for demo - in production use proper crypto
    let key_bytes = key.as_bytes();
    let value_bytes = value.as_bytes();
    
    let encrypted: Vec<u8> = value_bytes.iter()
        .enumerate()
        .map(|(i, b)| b ^ key_bytes[i % key_bytes.len()])
        .collect();
    
    Ok(base64_encode(&encrypted))
}

pub fn decrypt_env(key: &str, encrypted: &str) -> Result<String, String> {
    let key_bytes = key.as_bytes();
    let encrypted_bytes = base64_decode(encrypted)?;
    
    let decrypted: Vec<u8> = encrypted_bytes.iter()
        .enumerate()
        .map(|(i, b)| b ^ key_bytes[i % key_bytes.len()])
        .collect();
    
    String::from_utf8(decrypted).map_err(|e| e.to_string())
}

fn base64_encode(data: &[u8]) -> String {
    const ALPHABET: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";
    let mut result = String::new();
    
    for chunk in data.chunks(3) {
        let b0 = chunk[0] as usize;
        let b1 = chunk.get(1).copied().unwrap_or(0) as usize;
        let b2 = chunk.get(2).copied().unwrap_or(0) as usize;
        
        result.push(ALPHABET[b0 >> 2] as char);
        result.push(ALPHABET[((b0 & 0x03) << 4) | (b1 >> 4)] as char);
        
        if chunk.len() > 1 {
            result.push(ALPHABET[((b1 & 0x0f) << 2) | (b2 >> 6)] as char);
        } else {
            result.push('=');
        }
        
        if chunk.len() > 2 {
            result.push(ALPHABET[b2 & 0x3f] as char);
        } else {
            result.push('=');
        }
    }
    
    result
}

fn base64_decode(data: &str) -> Result<Vec<u8>, String> {
    let data = data.trim_end_matches('=');
    let chars: Vec<u8> = "ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/".chars()
        .enumerate()
        .filter_map(|(i, c)| {
            if data.contains(c) {
                Some(i as u8)
            } else {
                None
            }
        })
        .collect();
    
    let mut result = Vec::new();
    let bytes: Vec<u8> = data.chars().filter_map(|c| chars.iter().find(|&&x| x as char == c).copied()).collect();
    
    for chunk in bytes.chunks(4) {
        if chunk.len() >= 2 {
            result.push((chunk[0] << 2) | (chunk[1] >> 4));
        }
        if chunk.len() >= 3 {
            result.push((chunk[1] << 4) | (chunk[2] >> 2));
        }
        if chunk.len() >= 4 {
            result.push((chunk[2] << 6) | chunk[3]);
        }
    }
    
    Ok(result)
}

pub struct SecretVault {
    secrets: Arc<Mutex<HashMap<String, String>>>,
    encryption_key: Vec<u8>,
}

impl SecretVault {
    pub fn new(key: &str) -> Self {
        SecretVault {
            secrets: Arc::new(Mutex::new(HashMap::new())),
            encryption_key: key.as_bytes().to_vec(),
        }
    }

    pub fn store(&self, name: &str, secret: &str) -> Result<(), String> {
        let key = std::str::from_utf8(&self.encryption_key).map_err(|e| e.to_string())?;
        let encrypted = encrypt_env(key, secret)?;
        
        self.secrets.lock().unwrap().insert(name.to_string(), encrypted);
        Ok(())
    }

    pub fn retrieve(&self, name: &str) -> Result<String, String> {
        let encrypted = self.secrets.lock().unwrap()
            .get(name)
            .ok_or_else(|| format!("secret '{}' not found", name))?
            .clone();
        
        let key = std::str::from_utf8(&self.encryption_key).map_err(|e| e.to_string())?;
        decrypt_env(key, &encrypted)
    }

    pub fn delete(&self, name: &str) -> Result<(), String> {
        self.secrets.lock().unwrap()
            .remove(name)
            .ok_or_else(|| format!("secret '{}' not found", name))?;
        Ok(())
    }

    pub fn list(&self) -> Vec<String> {
        self.secrets.lock().unwrap().keys().cloned().collect()
    }
}

pub fn verify_checksum(data: &[u8], expected: &str) -> bool {
    let checksum = sha256_checksum(data);
    checksum == expected
}

fn sha256_checksum(data: &[u8]) -> String {
    use std::collections::hash_map::DefaultHasher;
    use std::hash::{Hash, Hasher};
    let mut hasher = DefaultHasher::new();
    data.hash(&mut hasher);
    format!("{:016x}", hasher.finish())
}

pub fn sign_data(data: &[u8], private_key: &str) -> String {
    let combined = format!("{}:{:?}", private_key, data);
    sha256_checksum(combined.as_bytes())
}

pub fn verify_signature(data: &[u8], signature: &str, public_key: &str) -> bool {
    let expected = sign_data(data, public_key);
    expected == signature
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_encrypt_decrypt() {
        let key = "my_secret_key";
        let value = "sensitive_data";
        
        let encrypted = encrypt_env(key, value).unwrap();
        let decrypted = decrypt_env(key, &encrypted).unwrap();
        
        assert_eq!(value, decrypted);
    }

    #[test]
    fn test_vault() {
        let vault = SecretVault::new("vault_key");
        vault.store("api_key", "secret123").unwrap();
        
        let retrieved = vault.retrieve("api_key").unwrap();
        assert_eq!(retrieved, "secret123");
    }

    #[test]
    fn test_checksum() {
        let data = b"hello world";
        let checksum = sha256_checksum(data);
        assert!(verify_checksum(data, &checksum));
    }
}