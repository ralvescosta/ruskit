use crate::{errors::AuthError, types::TokenClaims};
use async_trait::async_trait;
use jsonwebtoken::{
    decode, decode_header,
    jwk::{AlgorithmParameters, JwkSet},
    Algorithm, DecodingKey, TokenData, Validation,
};
use opentelemetry::Context;
use serde_json::Value;
use std::{collections::HashMap, str::FromStr};
use tracing::error;

#[async_trait]
pub trait JwtManager: Send + Sync {
    async fn verify(&self, ctx: &Context, token: &str) -> Result<TokenClaims, AuthError>;

    fn decode_token(
        &self,
        token: &str,
        jwks: &JwkSet,
        aud: &str,
        iss: &str,
    ) -> Result<TokenData<HashMap<String, Value>>, AuthError> {
        let Ok(header) = decode_header(token) else {
            error!("failed to decoded token header");
            return Err(AuthError::InvalidToken(
                "failed to decoded token header".into(),
            ));
        };

        let Some(kid) = header.kid else {
            error!("token header without kid");
            return Err(AuthError::InvalidToken("token header without kid".into()));
        };

        let Some(jwk) = jwks.find(&kid) else {
            error!("wasn't possible to find the same token kid into jwks");
            return Err(AuthError::InvalidToken(
                "wasn't possible to find the same token kid into jwks".into(),
            ));
        };

        let AlgorithmParameters::RSA(rsa) = &jwk.algorithm else {
            error!("token hashed using other algorithm than RSA");
            return Err(AuthError::InvalidToken(
                "token hashed using other algorithm than RSA".into(),
            ));
        };

        let Ok(decoding_key) = DecodingKey::from_rsa_components(&rsa.n, &rsa.e) else {
            error!("failed to decode rsa components");
            return Err(AuthError::InvalidToken(
                "failed to decode rsa components".into(),
            ));
        };

        let Some(key_alg) = jwk.common.key_algorithm else {
            error!("jwk with no key algorithm");
            return Err(AuthError::InvalidToken("jwk with no key algorithm".into()));
        };

        let Ok(alg) = Algorithm::from_str(key_alg.to_string().as_str()) else {
            error!("algorithm provided by the JWK is not sported!");
            return Err(AuthError::InvalidToken(
                "algorithm provided by the JWK is not sported!".into(),
            ));
        };

        let mut validation = Validation::new(alg);

        //Validation::SubjectPresent
        validation.set_audience(&[aud]);
        validation.set_issuer(&[iss]);
        validation.validate_exp = true;

        match decode::<HashMap<String, Value>>(token, &decoding_key, &validation) {
            Ok(d) => Ok(d),
            Err(err) => {
                error!(error = err.to_string(), "token validation error");
                Err(AuthError::InvalidToken("token validation error".into()))
            }
        }
    }
}
