use crate::services::{Service, API};
use crate::utils::errors::HikyakuResult;
use crate::utils::oauth2::services::get_box_oauth2_secret;

pub struct BoxService(API);

impl Service for BoxService {
    fn new(client_id: &str, client_secret: &str, redirect_uri: Option<&str>) -> HikyakuResult<Self> {
        let secret =
            get_box_oauth2_secret(client_id, client_secret, redirect_uri)?;
        let api = API::new(secret, "https://api.box.com/2.0");

        Ok(Self(api))
    }
}