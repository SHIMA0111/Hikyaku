use std::fmt::Display;
use oauth2::AuthType;

#[derive(Copy, Clone, PartialEq, Eq, Debug)]
pub enum Oauth2Provider {
    Google,
    Microsoft,
    Box
}

const REQUEST_BODY_SECRET: [Oauth2Provider;1] = [Oauth2Provider::Box];

impl Oauth2Provider {
    pub(crate) fn auth_type(&self) -> AuthType {
        if REQUEST_BODY_SECRET.contains(&self) {
            AuthType::RequestBody
        } else {
            AuthType::BasicAuth
        }
    }
}

impl Display for Oauth2Provider {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Oauth2Provider::Google => write!(f, "Google"),
            Oauth2Provider::Microsoft => write!(f, "Microsoft"),
            Oauth2Provider::Box => write!(f, "Box"),
        }
    }
}
