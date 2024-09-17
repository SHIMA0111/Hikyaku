use std::collections::HashMap;
use std::sync::Arc;
use std::sync::atomic::AtomicBool;
use std::time::Duration;
use axum::extract::{Query, State};
use axum::response::Redirect;
use axum::Router;
use axum::routing::get;
use log::{debug, error, info};
use oauth2::{AuthorizationCode, CsrfToken, PkceCodeChallenge, PkceCodeVerifier, Scope, TokenResponse};
use oauth2::basic::BasicClient;
use oauth2::reqwest::async_http_client;
use serde::Deserialize;
use time::OffsetDateTime;
use tokio::sync::mpsc::{Sender};
use tokio::sync::Mutex;
use crate::utils::oauth2::Token;
use crate::utils::oauth2::drop_control::Defer;

#[derive(Clone)]
pub(crate) struct AppState {
    oauth_client: BasicClient,
    scopes: Vec<String>,
    pkce_verifier: Arc<Mutex<Option<PkceCodeVerifier>>>,
    csrf_token: Arc<Mutex<Option<CsrfToken>>>,
    shutdown_flag: Arc<AtomicBool>,
    extra_args: HashMap<String, String>,
    sender: Sender<Token>
}

#[derive(Deserialize, Debug)]
pub(crate) struct AuthCallback {
    code: Option<String>,
    state: Option<String>,
    error: Option<String>,
    error_description: Option<String>,
}

impl AuthCallback {
    fn code(&self) -> Option<&str> {
        self.code.as_ref().map(|code| code.as_str())
    }

    fn state(&self) -> String {
        self.state.as_ref()
            .map(|state| state.to_string())
            .unwrap_or("".to_string())
    }
}

pub(crate) async fn spawn_webserver(client: &BasicClient,
                                    scopes: &[&str],
                                    protocol: &str,
                                    redirect_hostname: &str,
                                    port: u16,
                                    init_path: &str,
                                    redirect_path: &str,
                                    extra_args: &HashMap<String, String>,
                                    sender: Sender<Token>) {
    let shutdown_flag = Arc::new(AtomicBool::new(false));

    let state = AppState {
        oauth_client: client.clone(),
        scopes: scopes.iter().map(|scope| scope.to_string()).collect(),
        pkce_verifier: Arc::new(Mutex::new(None)),
        csrf_token: Arc::new(Mutex::new(None)),
        shutdown_flag: shutdown_flag.clone(),
        extra_args: extra_args.clone(),
        sender,
    };

    let auth_init_uri = if [443, 80].contains(&port) {
        format!("{}://{}{}", protocol, redirect_hostname, init_path)
    } else {
        format!("{}://{}:{}{}", protocol, redirect_hostname, port, init_path)
    };
    println!("Please access and auth this app: {}", auth_init_uri);

    let app = Router::new()
        .route(init_path, get(init_auth))
        .route(redirect_path, get(callback_auth))
        .route("/auth/complete", get(complete_auth2))
        .route("/auth/failed", get(failed_auth2))
        .route("/auth/infringe", get(infringed_connection))
        .with_state(state);

    let listener = tokio::net::TcpListener::bind(format!("0.0.0.0:{}", port)).await.unwrap();
    info!("Start oauth2.0 web server.");

    if let Err(e) = axum::serve(listener, app).with_graceful_shutdown(shutdown(shutdown_flag)).await {
        error!("Server error: {}", e);
    };
    info!("Shutting down oauth2.0 web server by finish signal of oauth2.0");
}

async fn shutdown(shutdown_flag: Arc<AtomicBool>) {
    while !shutdown_flag.load(std::sync::atomic::Ordering::Relaxed) {
        tokio::time::sleep(Duration::from_secs(1)).await;
    }
}

pub(crate) async fn init_auth(State(state): State<AppState>) -> Redirect {
    let mut auth_url = state.oauth_client.authorize_url(CsrfToken::new_random);
    for scope in &state.scopes {
        auth_url = auth_url.add_scope(Scope::new(scope.to_string()));
    }
    for (key, value) in &state.extra_args {
        auth_url = auth_url.add_extra_param(key, value);
    }
    let (pkce_code_challenge, pkce_code_verifier) = PkceCodeChallenge::new_random_sha256();
    let (authorization_uri, csrf_token) = auth_url
        .set_pkce_challenge(pkce_code_challenge)
        .url();
    *state.csrf_token.lock().await = Some(csrf_token);
    *state.pkce_verifier.lock().await = Some(pkce_code_verifier);

    Redirect::to(authorization_uri.as_str())
}

pub(crate) async fn callback_auth(Query(auth_callback): Query<AuthCallback>,
                                  State(state): State<AppState>) -> Redirect {
    let _server_drop = Defer::new(|| {
        debug!("Dropping oauth2 server by Defer.");
        state.shutdown_flag.store(true, std::sync::atomic::Ordering::Relaxed);
    });

    let code = auth_callback.code();
    let stored_csrf_token = state.csrf_token.lock().await.take();
    let csrf_state = CsrfToken::new(auth_callback.state());
    if stored_csrf_token.is_none() {
        error!("csrf_state cannot be None. Please contact developers.");
        return Redirect::to("/auth/failed");
    }

    if csrf_state.secret() == stored_csrf_token.unwrap().secret() {
        let code = match code {
            Some(code) => code,
            None => {
                debug!("callback query: {:?}", auth_callback);
                error!("Authorization code indicates empty.");
                return if let Some(error) = auth_callback.error.as_ref() {
                    let message =
                        format!("Failed reason: {} ({})",
                                error,
                                auth_callback.error_description
                                    .as_ref()
                                    .map(String::as_str)
                                    .unwrap_or("Unknown details"));
                    Redirect::to(format!("/auth/failed?message={}", message).as_str())
                } else {
                    Redirect::to("/auth/failed")
                }

            }
        };

        let pkce_code_verifier = state.pkce_verifier.lock().await.take();

        let token_result = match pkce_code_verifier {
            Some(verifier) => {
                state.oauth_client
                    .exchange_code(AuthorizationCode::new(code.to_string()))
                    .set_pkce_verifier(verifier)
                    .request_async(async_http_client)
                    .await
            },
            None => {
                error!("Failed to fetch PKCE verifier. PKCE required due to security");
                return Redirect::to("/auth/failed");
            },
        };

        match token_result {
            Ok(token) => {
                let expires_in = token.expires_in().unwrap_or(Duration::from_secs(3600));
                let expires_at = OffsetDateTime::now_utc() + expires_in;

                let token_data = Token {
                    application_id: state.oauth_client.client_id().to_string(),
                    access_token: token.access_token().secret().to_string(),
                    refresh_token: token.refresh_token().map(|refresh| refresh.secret().to_string()),
                    expires_at,
                    scopes: state.scopes.clone()
                };
                info!("Complete get signature.");

                if let Err(e) = state.sender.send(token_data).await {
                    error!("Failed to send token data: {:?}", e);
                    return Redirect::to("/auth/failed")
                };
                Redirect::to("/auth/complete")
            },
            Err(e) => {
                error!("Failed to fetch access token: {:?}", e);
                Redirect::to("/auth/failed")
            },
        }
    } else {
        error!("CSRF state is not valid. This connection may be infringed.");
        Redirect::to("/auth/infringe")
    }
}

pub(crate) async fn complete_auth2() -> &'static str {
    "Authentication successful. Please return your application."
}

#[derive(Deserialize)]
pub(crate) struct ErrorMessage {
    message: Option<String>,
}

pub(crate) async fn failed_auth2(Query(error): Query<ErrorMessage>) -> String {
    match error.message {
        Some(message) => {
            error!("Failed auth2 with error: {}", message);
            format!("Authentication failed.\nError: {}", message)
        },
        None => {
            "Authentication failed.".to_string()
        }
    }
}

pub(crate) async fn infringed_connection() -> &'static str {
    "Csrf token verification failed. This connection may be infringed."
}
