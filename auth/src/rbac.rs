use crate::types::TokenClaims;

pub fn is_allowed<'i>(claims: &'i TokenClaims, permission: &'i str) -> bool {
    let Some(_) = claims.scopes.0.get(permission) else {
        return false;
    };

    true
}
