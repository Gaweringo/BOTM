use std::{collections::HashMap, net::TcpListener};

use base64::{engine::general_purpose, Engine};
use color_eyre::eyre::bail;
use log::info;
use rspotify::{scopes, AuthCodeSpotify, Config, Credentials, OAuth};
use serde::{Deserialize, Serialize};
use spotify_oauth::generate_random_string;

#[derive(thiserror::Error, Debug)]
pub enum ConfigError {
    #[error("Client ID is not correct")]
    BadClientId,
    #[error("Client secret does not work")]
    BadClientSecret,
    #[error("This port can not be used")]
    BadPort {
        // #[from]
        source: std::io::Error,
        port: i32,
    },
    #[error(transparent)]
    ClientError(#[from] rspotify::ClientError),
    #[error(transparent)]
    ReqwestError(#[from] reqwest::Error),
    #[error("Unknown error")]
    Unknown,
}

pub fn check_config(client_id: &str, client_secret: &str, port: i32) -> Result<(), ConfigError> {
    log::info!("Checking config");

    // May require the `env-file` feature enabled if the environment variables
    // aren't configured manually.
    let config = Config {
        // token_refreshing: true,
        // token_cached: true,
        ..Default::default()
    };

    let creds = Credentials::new(client_id, client_secret);

    let mut spotify = rspotify::ClientCredsSpotify::with_config(creds, config);
    // spotify.request_token()?;

    // TODO: Find out if client id or client secret is wrong
    let res = reqwest::blocking::Client::new()
        .post("https://accounts.spotify.com/api/token")
        .header(
            "Authorization",
            format!(
                "Basic {}",
                general_purpose::STANDARD.encode(format!("{client_id}:{client_secret}"))
            ),
        )
        .form(&HashMap::from([("grant_type", "client_credentials")]))
        .send()?;

    // info!("Res text: {:?}", res.text());

    let res: TokenResponse = res.json()?;

    log::info!("Res: {:#?}", res);

    match res {
        TokenResponse::ErrorResponse(ErrorResponse{error: _, error_description: err_des}) => match err_des.as_str() {
            "Invalid client" => return Err(ConfigError::BadClientId),
            "Invalid client secret" => return Err(ConfigError::BadClientSecret),
            _ => return Err(ConfigError::Unknown)
            
        },
        TokenResponse::SuccessfulResponse(_) => {}
    }

    let _tcp = TcpListener::bind(format!("127.0.0.1:{}", port))
        .map_err(|source| ConfigError::BadPort { source, port })?;
    Ok(())
}

#[derive(Serialize, Deserialize, Debug)]
struct ErrorResponse {
    error: String,
    error_description: String,
}

#[derive(Serialize, Deserialize, Debug)]
struct SuccessfulResponse {
    access_token: String,
    token_type: String,
    expires_in: i32,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(untagged)]
enum TokenResponse {
    ErrorResponse(ErrorResponse),
    SuccessfulResponse(SuccessfulResponse),
}
