use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;
use tracing::error;

#[derive(Debug, Default, Serialize, Deserialize, Clone)]
pub struct CustomData {
    #[serde(rename(serialize = "x1", deserialize = "x1"))]
    pub user_id: i64,

    #[serde(rename(serialize = "x2", deserialize = "x2"))]
    pub user_key: String,

    #[serde(rename(serialize = "y1", deserialize = "y1"))]
    pub company_id: i64,

    #[serde(rename(serialize = "y2", deserialize = "y2"))]
    pub company_key: String,
}

impl CustomData {
    pub fn from_auth0(claims: &HashMap<String, Value>) -> Result<Self, ()> {
        let Some(user_data) = claims.get("user_data") else {
            error!(claim = "user_data", "invalid jwt claim");
            return Err(());
        };

        let Some(user_metadata) = user_data.get("user_metadata") else {
            error!(claim = "user_metadata", "invalid jwt claim");
            return Err(());
        };

        let Some(x1) = user_metadata.get("x1") else {
            error!(claim = "x1", "invalid jwt claim");
            return Err(());
        };

        let Some(x2) = user_metadata.get("x2") else {
            error!(claim = "x2", "invalid jwt claim");
            return Err(());
        };

        let Some(y1) = user_metadata.get("y1") else {
            error!(claim = "y1", "invalid jwt claim");
            return Err(());
        };

        let Some(y2) = user_metadata.get("y2") else {
            error!(claim = "y2", "invalid jwt claim");
            return Err(());
        };

        Ok(Self {
            user_id: x1.as_i64().unwrap(),
            user_key: x2.as_str().unwrap().into(),
            company_id: y1.as_i64().unwrap(),
            company_key: y2.as_str().unwrap().into(),
        })
    }

    pub fn from_keycloak(claims: &HashMap<String, Value>) -> Result<Self, ()> {
        let Some(user_data) = claims.get("user_data") else {
            error!(claim = "user_data", "invalid jwt claim");
            return Err(());
        };

        let Some(json) = user_data.as_str() else {
            error!(claim = "user_data", "invalid jwt claim");
            return Err(());
        };

        let Ok(custom) = serde_json::from_str::<CustomData>(json) else {
            error!(claim = "user_data", "invalid jwt claim");
            return Err(());
        };

        Ok(custom)
    }
}
