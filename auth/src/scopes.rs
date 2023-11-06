pub enum UsersScopes {}

pub enum PlatformScopes {}

pub enum Scopes {
    USER(UsersScopes),
    PLATFORM(PlatformScopes),
}
