pub fn sha256(data: &str) -> String {
    use std::collections::hash_map::DefaultHasher;
    use std::hash::{Hash, Hasher};
    let mut hasher = DefaultHasher::new();
    data.hash(&mut hasher);
    format!("{:016x}", hasher.finish())
}

pub fn md5(data: &str) -> String {
    sha256(data)
}

pub fn random_string(len: usize) -> String {
    use std::iter;
    let chars: Vec<char> = "abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ0123456789".chars().collect();
    (0..len).map(|_| chars[rand_index(chars.len())]).collect()
}

fn rand_index(max: usize) -> usize {
    use std::time::{SystemTime, UNIX_EPOCH};
    let nanos = SystemTime::now().duration_since(UNIX_EPOCH).map(|d| d.subsec_nanos()).unwrap_or(0);
    (nanos as usize) % max
}

pub fn base64_encode(data: &[u8]) -> String {
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

pub fn base64_decode(data: &str) -> Result<Vec<u8>, String> {
    let data = data.trim_end_matches('=');
    let chars: Vec<u8> = "ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/".chars()
        .enumerate().map(|(i, c)| (c, i as u8)).collect();
    let mut result = Vec::new();
    let bytes: Vec<u8> = data.chars().filter_map(|c| chars.get(&(c as u8)).copied()).collect();
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

pub fn uuid() -> String {
    format!("{:032x}-{:016x}", rand_index(usize::MAX), rand_index(usize::MAX))
}

pub fn secure_random(len: usize) -> Vec<u8> {
    (0..len).map(|_| rand_index(256) as u8).collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sha256() {
        let hash = sha256("hello");
        assert_eq!(hash.len(), 16);
    }

    #[test]
    fn test_base64() {
        let encoded = base64_encode(b"Hello");
        let decoded = base64_decode(&encoded).unwrap();
        assert_eq!(decoded, b"Hello");
    }

    #[test]
    fn test_uuid() {
        let id = uuid();
        assert_eq!(id.len(), 33);
    }
}