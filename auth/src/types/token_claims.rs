use super::{CustomData, Scopes};
use crate::errors::AuthError;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;
use tracing::error;

#[derive(Debug, Default, Serialize, Deserialize, Clone)]
pub struct TokenClaims {
    pub iss: String,
    pub sub: String,
    pub aud: Vec<String>,
    pub iat: u64,
    pub exp: u64,
    pub scopes: Scopes,
    pub custom_data: CustomData,
}

impl TokenClaims {
    fn new(
        claims: &HashMap<String, Value>,
        scopes: Scopes,
        custom_data: CustomData,
    ) -> Result<Self, AuthError> {
        Ok(Self {
            iss: get_claim_as_string("iss", claims)?,
            sub: get_claim_as_string("sub", claims)?,
            aud: get_claim_as_vec("aud", claims)?,
            iat: get_claim_as_u64("iat", claims)?,
            exp: get_claim_as_u64("exp", claims)?,
            scopes,
            custom_data,
        })
    }

    pub fn from_auth0(claims: &HashMap<String, Value>) -> Result<Self, AuthError> {
        let scopes = Scopes::from_auth0(claims)?;
        let custom_data = CustomData::from_auth0(claims)?;
        TokenClaims::new(claims, scopes, custom_data)
    }

    pub fn from_keycloak(claims: &HashMap<String, Value>) -> Result<Self, AuthError> {
        let scopes = Scopes::from_keycloak(claims)?;
        let custom_data = CustomData::from_keycloak(claims)?;
        TokenClaims::new(claims, scopes, custom_data)
    }
}

fn get_claim_as_string(key: &str, claims: &HashMap<String, Value>) -> Result<String, AuthError> {
    let Some(fv) = claims.get(key) else {
        error!(claim = key, "invalid jwt claim");
        return Err(AuthError::FailedToRetrieveClaim);
    };

    let Some(value) = fv.as_str() else {
        error!(claim = key, "invalid jwt claim");
        return Err(AuthError::FailedToRetrieveClaim);
    };

    Ok(value.into())
}

fn get_claim_as_u64(key: &str, claims: &HashMap<String, Value>) -> Result<u64, AuthError> {
    let Some(fv) = claims.get(key) else {
        error!(claim = key, "invalid jwt claim");
        return Err(AuthError::FailedToRetrieveClaim);
    };

    let Some(value) = fv.as_u64() else {
        error!(claim = key, "invalid jwt claim");
        return Err(AuthError::FailedToRetrieveClaim);
    };

    Ok(value)
}

fn get_claim_as_vec(key: &str, claims: &HashMap<String, Value>) -> Result<Vec<String>, AuthError> {
    let Some(fv) = claims.get(key) else {
        error!(claim = key, "invalid jwt claim");
        return Err(AuthError::FailedToRetrieveClaim);
    };

    let Some(value) = fv.as_array() else {
        error!(claim = key, "invalid jwt claim");
        return Err(AuthError::FailedToRetrieveClaim);
    };

    Ok(value
        .iter()
        .map(|v| v.as_str().unwrap().to_owned())
        .collect::<Vec<String>>())
}
