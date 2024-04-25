#[derive(Debug, Clone)]
pub struct IdentityServerConfigs {
    /// Identity Server URL
    ///
    /// Default: ""
    pub url: String,
    /// Identity Application Realm
    ///
    /// in Auth0 Realm is the same than Domain
    pub realm: String,
    //Default: ""
    pub audience: String,
    //Default: ""
    pub issuer: String,
    //Default: ""
    pub client_id: String,
    //Default: ""
    pub client_secret: String,
    //Default: "client_credentials"
    pub grant_type: String,
}

impl Default for IdentityServerConfigs {
    fn default() -> Self {
        Self {
            url: Default::default(),
            realm: Default::default(),
            audience: Default::default(),
            issuer: Default::default(),
            client_id: Default::default(),
            client_secret: Default::default(),
            grant_type: "client_credentials".to_owned(),
        }
    }
}
