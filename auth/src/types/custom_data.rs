use crate::errors::AuthError;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;
use tracing::error;

#[derive(Debug, Default, Serialize, Deserialize, Clone)]
pub struct CustomData {
    pub user_data: Value,
    pub user_metadata: Value,
}

impl CustomData {
    pub fn from_auth0(claims: &HashMap<String, Value>) -> Result<Self, AuthError> {
        let Some(user_data) = claims.get("user_data") else {
            error!(claim = "user_data", "invalid jwt claim");
            return Err(AuthError::FailedToRetrieveUserCustomDataClaim);
        };

        let Some(user_metadata) = user_data.get("user_metadata") else {
            error!(claim = "user_metadata", "invalid jwt claim");
            return Err(AuthError::FailedToRetrieveUserCustomDataClaim);
        };

        Ok(Self {
            user_data: user_data.to_owned(),
            user_metadata: user_metadata.to_owned(),
        })
    }

    pub fn from_keycloak(claims: &HashMap<String, Value>) -> Result<Self, AuthError> {
        let Some(user_data) = claims.get("user_data") else {
            error!(claim = "user_data", "invalid jwt claim");
            return Err(AuthError::FailedToRetrieveUserCustomDataClaim);
        };

        let Some(json) = user_data.as_str() else {
            error!(claim = "user_data", "invalid jwt claim");
            return Err(AuthError::FailedToRetrieveUserCustomDataClaim);
        };

        let Ok(custom) = serde_json::from_str::<CustomData>(json) else {
            error!(claim = "user_data", "invalid jwt claim");
            return Err(AuthError::FailedToRetrieveUserCustomDataClaim);
        };

        Ok(custom)
    }
}
