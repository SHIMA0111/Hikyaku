use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};
use log::debug;
use time::OffsetDateTime;
use crate::utils::oauth2::Token;

pub(crate) fn save_token<TS>(service: TS, token: &Token, token_path: &Path) -> std::io::Result<()>
where
    TS: ToString
{
    let (path, mut saved_tokens) = _load_token(token_path);
    if saved_tokens.len() > 0 {
        debug!("Found token file. Add the new token in it");
    }
    saved_tokens.retain(|service_str, token| {
        service.to_string() != *service_str &&
            (token.expires_at > OffsetDateTime::now_utc() || token.refresh_token.is_some())
    });
    saved_tokens.insert(service.to_string(), token.clone());
    let token_json = serde_json::to_string(&saved_tokens)?;

    if let Some(dir) = path.as_path().parent() {
        if !dir.exists() {
            debug!("Creating directory {}", dir.display());
            fs::create_dir_all(dir)?;
        }
    }

    fs::write(path, token_json)
}

pub(crate) fn load_token<TS>(service: TS, token_path: &Path) -> Option<Token>
where
    TS: ToString
{
    let (_, tokens) = _load_token(token_path);
    debug!("Loaded token number: {}", tokens.len());
    tokens.get(&service.to_string()).map(|token| token.clone())
}

fn _load_token(token_path: &Path) -> (PathBuf, HashMap<String, Token>) {
    let mut token_path = token_path.to_path_buf();
    token_path.push("tokens.json");
    if token_path.exists() {
        debug!("Token file found at {:?}", token_path);
        match fs::read_to_string(&token_path) {
            Ok(token) => (token_path, serde_json::from_str(&token).unwrap_or(HashMap::new())),
            Err(_) => (token_path, HashMap::new()),
        }
    } else {
        (token_path, HashMap::new())
    }
}