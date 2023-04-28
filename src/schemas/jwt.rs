use crate::{
    app::TokenConfig,
    errors::{Auth, Error, ErrorTypes},
};
use actix_web::{HttpMessage, HttpRequest};

use actix_web::cookie::time::Duration as ActixWebDuration;
use jsonwebtoken::{DecodingKey, EncodingKey, Header, Validation};
use serde::{Deserialize, Serialize};

use time::{Duration, OffsetDateTime};
use tracing::instrument;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct TokenClaims {
    pub sub: String,
    pub exp: usize,
}

impl TokenClaims {
    pub fn new(sub: String, time: Duration) -> Self {
        //create token
        let now = OffsetDateTime::now_utc();
        let exp = (now + time).unix_timestamp() as usize;
        TokenClaims { sub, exp }
    }
}
pub struct TokenSettings {
    decode_key: DecodingKey,
    encode_key: EncodingKey,
    pub exp: Duration,
    pub maxage: ActixWebDuration,
}

pub struct Jwt {
    pub access: TokenSettings,
    pub refresh: TokenSettings,
}

pub enum TokenType {
    Refresh,
    Access,
}

impl Jwt {
    pub fn new(access_config: &TokenConfig, refresh_config: &TokenConfig) -> Self {
        Jwt {
            access: TokenSettings {
                decode_key: DecodingKey::from_secret(access_config.key.as_bytes()),
                encode_key: EncodingKey::from_secret(access_config.key.as_bytes()),
                exp: Duration::minutes(access_config.exp),
                maxage: ActixWebDuration::new(60 * access_config.maxage, 0),
            },
            refresh: TokenSettings {
                decode_key: DecodingKey::from_secret(refresh_config.key.as_bytes()),
                encode_key: EncodingKey::from_secret(refresh_config.key.as_bytes()),
                exp: Duration::minutes(refresh_config.exp),
                maxage: ActixWebDuration::new(60 * refresh_config.maxage, 0),
            },
        }
    }

    pub fn encode(&self, token: &TokenClaims, token_type: TokenType) -> Result<String, Error> {
        tracing::info!("JWT token encoding");
        let token =
            match token_type {
                TokenType::Access => {
                    jsonwebtoken::encode(&Header::default(), token, &self.access.encode_key)
                        .map_err(|e| Error {
                            cause: Some(e.to_string()),
                            message: Some("Can not create token".into()),
                            error_type: ErrorTypes::JwtError,
                        })?
                }
                TokenType::Refresh => {
                    jsonwebtoken::encode(&Header::default(), token, &self.refresh.encode_key)
                        .map_err(|e| Error {
                            cause: Some(e.to_string()),
                            message: Some("Can not create token".into()),
                            error_type: ErrorTypes::JwtError,
                        })?
                }
            };

        Ok(token)
    }

    pub fn decode(&self, claim: &str, token_type: TokenType) -> Result<TokenClaims, Error> {
        tracing::info!("JWT token decoding");
        let token = match token_type {
            TokenType::Access => jsonwebtoken::decode::<TokenClaims>(
                claim,
                &self.access.decode_key,
                &Validation::new(jsonwebtoken::Algorithm::HS256),
            )
            .map_err(|e| Error {
                cause: Some(e.to_string()),
                message: Some("Can not decode token".into()),
                error_type: ErrorTypes::JwtError,
            })?,
            TokenType::Refresh => jsonwebtoken::decode::<TokenClaims>(
                claim,
                &self.refresh.decode_key,
                &Validation::new(jsonwebtoken::Algorithm::HS256),
            )
            .map_err(|e| Error {
                cause: Some(e.to_string()),
                message: Some("Can not decode token".into()),
                error_type: ErrorTypes::JwtError,
            })?,
        };
        Ok(token.claims)
    }

    #[instrument(skip_all, name = "Refresh jwt token")]
    pub fn refresh(&self, req: &HttpRequest) -> Result<uuid::Uuid, Error> {
        tracing::info!("Get jwt refresh token from cookies");

        let tokens = match req.cookie("refresh_token").map(|c| c.value().to_string()) {
            Some(token) => token,
            None => {
                tracing::error!("JWT refresh token not found");
                return Err(Error {
                    cause: None,
                    message: Some("Refresh jwt token not found. Log in first!".into()),
                    error_type: ErrorTypes::Auth(Auth::Authorization),
                });
            }
        };

        let token = match self.decode(&tokens, TokenType::Refresh) {
            Ok(c) => c,
            Err(e) => {
                tracing::error!("Invalid refresh jwt token");

                return Err(Error {
                    cause: Some(e.to_string()),
                    message: Some("Invalid refresh jwt tokens".into()),
                    error_type: ErrorTypes::JwtError,
                });
            }
        };

        //check if token is valid
        if OffsetDateTime::now_utc().unix_timestamp() as usize > token.exp {
            tracing::error!("Refresh token timed out");

            return Err(Error {
                cause: None,
                message: Some("Refresh token timed out".into()),
                error_type: ErrorTypes::Auth(Auth::Authorization),
            });
        }

        //Get user id
        let req_ext = req.extensions();

        let user_id = req_ext.get::<uuid::Uuid>().ok_or(Error {
            cause: None,
            message: Some("Can not find user's id".into()),
            error_type: ErrorTypes::Auth(Auth::Authentication),
        })?;

        Ok(user_id.clone())
    }
}
