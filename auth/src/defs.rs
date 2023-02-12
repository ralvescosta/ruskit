pub enum UsersScopes {}

pub enum ThingsScopes {}

pub enum PlatformScopes {}

pub enum Scopes {
    USER(UsersScopes),
    THING(ThingsScopes),
    PLATFORM(PlatformScopes),
}
