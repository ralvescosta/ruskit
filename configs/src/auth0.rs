#[derive(Debug, Clone)]
pub struct Auth0Configs {
    //Default: ""
    pub domain: String,
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

impl Default for Auth0Configs {
    fn default() -> Self {
        Self {
            domain: Default::default(),
            audience: Default::default(),
            issuer: Default::default(),
            client_id: Default::default(),
            client_secret: Default::default(),
            grant_type: "client_credentials".to_owned(),
        }
    }
}
