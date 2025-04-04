use std::fmt::Display;
use oauth2::AuthType;


/// Represents the different types of OAuth authentication methods.
///
/// From [RFC6749](https://datatracker.ietf.org/doc/html/rfc6749#section-2.3.1),
/// the client secret can be RequestBody or BasicAuth(recommended).
///
/// This variant supports the two type of it. This follows [`AuthType`]
/// and converted to it internally.
#[derive(Copy, Clone, PartialEq, Eq, Debug)]
pub enum OauthType {
    RequestBody,
    BasicAuth,
}

impl OauthType {
    fn convert_oauth2_type(&self) -> AuthType {
        match self {
            Self::RequestBody => AuthType::RequestBody,
            Self::BasicAuth => AuthType::BasicAuth,
        }
    }
}

#[derive(Clone, PartialEq, Eq, Debug)]
pub(crate) enum Oauth2Provider {
    Google,
    Microsoft,
    Box,
    Dropbox,
    Custom{
        name: String,
        auth_type: OauthType,
    },
}

const REQUEST_BODY_SECRET: [Oauth2Provider;1] = [Oauth2Provider::Box];

impl Oauth2Provider {
    pub(crate) fn auth_type(&self) -> AuthType {
        match self {
            Self::Custom {
                auth_type,
                ..
            } => auth_type.convert_oauth2_type(),
            _ => {
                if REQUEST_BODY_SECRET.contains(&self) {
                    AuthType::RequestBody
                } else {
                    AuthType::BasicAuth
                }
            }
        }
    }
}

impl Display for Oauth2Provider {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Oauth2Provider::Google => write!(f, "Google"),
            Oauth2Provider::Microsoft => write!(f, "Microsoft"),
            Oauth2Provider::Box => write!(f, "Box"),
            Oauth2Provider::Dropbox => write!(f, "Dropbox"),
            Oauth2Provider::Custom{
                name,
                ..
            } => write!(f, "{}", name),
        }
    }
}
