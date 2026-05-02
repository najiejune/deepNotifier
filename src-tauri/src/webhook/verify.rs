use hmac::{Hmac, Mac};
use sha2::Sha256;

type HmacSha256 = Hmac<Sha256>;

pub fn verify_github_signature(secret: &str, payload: &str, signature: &str) -> bool {
    let expected = signature.strip_prefix("sha256=").unwrap_or(signature);
    let expected_bytes = match hex::decode(expected) {
        Ok(b) => b,
        Err(_) => return false,
    };

    let mut mac = match HmacSha256::new_from_slice(secret.as_bytes()) {
        Ok(m) => m,
        Err(_) => return false,
    };
    mac.update(payload.as_bytes());

    match mac.verify_slice(&expected_bytes) {
        Ok(_) => true,
        Err(_) => false,
    }
}

pub fn verify_gitlab_token(configured_token: &str, received_token: &str) -> bool {
    configured_token == received_token
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_verify_github_signature() {
        let secret = "mysecret";
        let payload = r#"{"test": true}"#;

        let mut mac = HmacSha256::new_from_slice(secret.as_bytes()).unwrap();
        mac.update(payload.as_bytes());
        let result = mac.finalize();
        let sig = hex::encode(result.into_bytes());

        assert!(verify_github_signature(secret, payload, &format!("sha256={}", sig)));
        assert!(!verify_github_signature(secret, payload, "sha256=invalid"));
    }

    #[test]
    fn test_verify_gitlab_token() {
        assert!(verify_gitlab_token("mytoken", "mytoken"));
        assert!(!verify_gitlab_token("mytoken", "wrongtoken"));
    }
}
