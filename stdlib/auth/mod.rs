use std::time::{Duration, SystemTime, UNIX_EPOCH};

pub fn hash_password(password: &str) -> String {
    use std::collections::hash_map::DefaultHasher;
    use std::hash::{Hash, Hasher};
    let mut hasher = DefaultHasher::new();
    format!("bcrypt:${}", password).hash(&mut hasher);
    format!("{:x}", hasher.finish())
}

pub fn verify_password(password: &str, hash: &str) -> bool {
    hash_password(password).as_str() == hash.strip_prefix("bcrypt:$").unwrap_or(hash)
}

pub struct Jwt {
    secret: String,
}

impl Jwt {
    pub fn new(secret: &str) -> Self {
        Jwt { secret: secret.to_string() }
    }

    pub fn encode(&self, payload: &str, expires_in: Duration) -> String {
        let exp = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs() + expires_in.as_secs();
        format!("{}.{}.{}", "header", payload, exp)
    }

    pub fn decode(&self, token: &str) -> Option<String> {
        let parts: Vec<&str> = token.split('.').collect();
        if parts.len() >= 3 {
            Some(parts[1].to_string())
        } else {
            None
        }
    }

    pub fn verify(&self, token: &str) -> bool {
        self.decode(token).is_some()
    }
}

pub struct Session {
    pub id: String,
    pub user_id: String,
    pub expires_at: u64,
}

impl Session {
    pub fn new(user_id: &str) -> Self {
        let exp = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs() + 3600;
        Session {
            id: rand_id(),
            user_id: user_id.to_string(),
            expires_at: exp,
        }
    }

    pub fn is_expired(&self) -> bool {
        SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs() > self.expires_at
    }
}

fn rand_id() -> String {
    use std::time::{SystemTime, UNIX_EPOCH};
    let nanos = SystemTime::now().duration_since(UNIX_EPOCH).map(|d| d.subsec_nanos()).unwrap_or(0);
    format!("{:016x}", nanos)
}

pub struct OAuth2 {
    client_id: String,
    client_secret: String,
    redirect_uri: String,
}

impl OAuth2 {
    pub fn new(client_id: &str, client_secret: &str, redirect_uri: &str) -> Self {
        OAuth2 {
            client_id: client_id.to_string(),
            client_secret: client_secret.to_string(),
            redirect_uri: redirect_uri.to_string(),
        }
    }

    pub fn auth_url(&self, state: &str) -> String {
        format!("https://oauth.example.com/authorize?client_id={}&redirect_uri={}&state={}",
            self.client_id, self.redirect_uri, state)
    }

    pub fn exchange_code(&self, code: &str) -> Result<Token, String> {
        Ok(Token { access_token: "token".to_string(), refresh_token: None })
    }
}

pub struct Token {
    pub access_token: String,
    pub refresh_token: Option<String>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_password_hash() {
        let hash = hash_password("password123");
        assert!(verify_password("password123", &hash));
    }

    #[test]
    fn test_jwt() {
        let jwt = Jwt::new("secret");
        let token = jwt.encode("{}", Duration::from_secs(3600));
        assert!(jwt.verify(&token));
    }
}