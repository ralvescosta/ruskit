use crate::errors::AuthError;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::{HashMap, HashSet};
use tracing::error;

#[derive(Debug, Default, Serialize, Deserialize, Clone)]
pub struct Scopes(pub HashSet<String>);

impl Scopes {
    pub fn from_auth0(claims: &HashMap<String, Value>) -> Result<Self, AuthError> {
        let Some(v) = claims.get("scope") else {
            error!(claim = "scope", "invalid jwt claim");
            return Err(AuthError::FailedToRetrieveScopeClaim);
        };

        let Some(scopes) = v.as_str() else {
            error!(claim = "scope", "invalid jwt claim");
            return Err(AuthError::FailedToRetrieveScopeClaim);
        };

        let mut set = HashSet::new();

        for scope in scopes.split_whitespace() {
            set.insert(scope.into());
        }

        Ok(Self(set))
    }

    pub fn from_keycloak(claims: &HashMap<String, Value>) -> Result<Self, AuthError> {
        let Some(access) = claims.get("resource_access") else {
            error!(claim = "resource_access", "invalid jwt claim");
            return Err(AuthError::FailedToRetrieveScopeClaim);
        };

        let Some(azp_v) = claims.get("azp") else {
            error!(claim = "azp", "invalid jwt claim");
            return Err(AuthError::FailedToRetrieveScopeClaim);
        };

        let Some(azp) = azp_v.as_str() else {
            error!(claim = "azp", "invalid jwt claim");
            return Err(AuthError::FailedToRetrieveScopeClaim);
        };

        let Some(resources) = access.get(azp) else {
            error!(claim = azp, "invalid jwt claim");
            return Err(AuthError::FailedToRetrieveScopeClaim);
        };

        let Some(roles) = resources.get("roles") else {
            error!(claim = "roles", "invalid jwt claim");
            return Err(AuthError::FailedToRetrieveScopeClaim);
        };

        let mut set = HashSet::new();

        for role in roles.as_array().unwrap() {
            set.insert(role.as_str().unwrap().into());
        }

        Ok(Self(set))
    }
}
