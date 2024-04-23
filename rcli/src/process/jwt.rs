use anyhow::Result;
use jsonwebtoken::{decode, encode, Algorithm, DecodingKey, EncodingKey, Header, Validation};
use serde::{Deserialize, Serialize};

use crate::{current_timestamp_sec, parse_str_to_timestamp};

#[derive(Debug, Serialize, Deserialize)]
struct Claims {
    #[serde(skip_serializing_if = "Option::is_none")]
    aud: Option<String>, // Optional. Audience
    #[serde(skip_serializing_if = "Option::is_none")]
    sub: Option<String>, // Optional. Subject (whom token refers to)
    exp: i64, // Required (validate_exp defaults to true in validation). Expiration time (as UTC timestamp)
    iat: i64,
}

impl Claims {
    pub fn new(aud: Option<String>, sub: Option<String>, exp: i64, iat: i64) -> Self {
        Self { aud, exp, sub, iat }
    }
}

const SECRET: &str = "your-256-bit-secret";

pub async fn process_jwt_sign(
    aud: Option<String>,
    sub: Option<String>,
    exp: String,
) -> Result<String> {
    let iat = current_timestamp_sec();
    let exp = parse_str_to_timestamp(&exp)?;
    let claims = Claims::new(aud, sub, exp, iat);
    let header = Header::new(Algorithm::HS256);
    let token = encode(&header, &claims, &EncodingKey::from_secret(SECRET.as_ref()))?;
    Ok(token)
}

pub async fn process_jwt_verify(token: String) -> Result<bool> {
    let mut validation = Validation::new(Algorithm::HS256);
    validation.validate_aud = false;
    let token = decode::<Claims>(
        &token,
        &DecodingKey::from_secret(SECRET.as_ref()),
        &validation,
    )?;

    let claims = token.claims;
    let exp = current_timestamp_sec();
    // token 过期时间大于 当前时间 返回 true
    let result = claims.exp > exp;
    Ok(result)
}

#[cfg(test)]
mod tests {
    use tokio::time::{sleep, Duration};

    use super::*;

    #[tokio::test]
    async fn test_jwt_sign_and_verify() {
        let token = process_jwt_sign(
            Some("abc".into()),
            Some("sub.sasdf".into()),
            "5s".to_string(),
        )
        .await
        .unwrap();

        dbg!(&token);

        let result = process_jwt_verify(token.clone()).await.unwrap();
        assert!(result);

        sleep(Duration::from_secs(5)).await;

        let result = process_jwt_verify(token).await.unwrap();
        assert!(!result);
    }
}
