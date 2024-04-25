use crate::types::TokenClaims;
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
    async fn verify(&self, ctx: &Context, token: &str) -> Result<TokenClaims, ()>;

    fn decode_token(
        &self,
        token: &str,
        jwks: &JwkSet,
        aud: &str,
        iss: &str,
    ) -> Result<TokenData<HashMap<String, Value>>, ()> {
        let Ok(header) = decode_header(token) else {
            error!("failed to decoded token header");
            return Err(());
        };

        let Some(kid) = header.kid else {
            error!("token header without kid");
            return Err(());
        };

        let Some(jwk) = jwks.find(&kid) else {
            error!("wasn't possible to find the same token kid into jwks");
            return Err(());
        };

        let AlgorithmParameters::RSA(rsa) = &jwk.algorithm else {
            error!("token hashed using other algorithm than RSA");
            return Err(());
        };

        let Ok(decoding_key) = DecodingKey::from_rsa_components(&rsa.n, &rsa.e) else {
            error!("failed to decode rsa components");
            return Err(());
        };

        let Some(key_alg) = jwk.common.key_algorithm else {
            error!("jwk with no key algorithm");
            return Err(());
        };

        let Ok(alg) = Algorithm::from_str(key_alg.to_string().as_str()) else {
            error!("algorithm provided by the JWK is not sported!");
            return Err(());
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
                Err(())
            }
        }
    }
}
