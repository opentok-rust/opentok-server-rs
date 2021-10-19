extern crate rustc_serialize;

use rand::Rng;
use rustc_serialize::hex::ToHex;
use serde::{Deserialize, Serialize};
use std::fmt;
use std::time::{SystemTime, UNIX_EPOCH};
use thiserror::Error;

mod http_client;

static SERVER_URL: &str = "https://api.opentok.com";
static API_ENDPOINT_PATH_START: &str = "/v2/project/";

/// Unique session identifier.
pub type SessionId = String;

/// OpenTokError enumerates all possible errors returned by this library.
#[derive(Debug, Error, PartialEq)]
pub enum OpenTokError {
    #[error("Bad request {0}")]
    BadRequest(String),
    #[error("Cannot encode request")]
    EncodingError,
    #[error("OpenTok server error {0}")]
    ServerError(String),
    #[error("Unexpected response {0}")]
    UnexpectedResponse(String),
    #[error("Unknown error")]
    __Unknown,
}

impl From<surf::Error> for OpenTokError {
    fn from(error: surf::Error) -> OpenTokError {
        match error.status().into() {
            400..=499 => OpenTokError::BadRequest(error.to_string()),
            500..=599 => OpenTokError::ServerError(error.to_string()),
            _ => OpenTokError::__Unknown,
        }
    }
}

/// Determines whether a session will transmit streams using the OpenTok Media Router
/// or not.
#[derive(Debug, PartialEq)]
pub enum MediaMode {
    /// The session will try to transmit streams directly between clients.
    Relayed,
    /// The session will transmit streams using the OpenTok Media Router.
    Routed,
}

impl fmt::Display for MediaMode {
    fn fmt(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        write!(formatter, "{}", format!("{:?}", self).to_lowercase())
    }
}

/// Determines whether a session is automatically archived or not.
/// Archiving is currently unsupported.
#[derive(Debug)]
pub enum ArchiveMode {
    /// The session will always be archived automatically.
    Always,
    /// A POST request to /archive is required to archive the session.
    /// Currently unsupported.
    Manual,
}

impl fmt::Display for ArchiveMode {
    fn fmt(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        write!(formatter, "{}", format!("{:?}", self).to_lowercase())
    }
}

/// OpenTok Session options to be provided at Session creation time.
#[derive(Default)]
pub struct SessionOptions<'a> {
    /// An IP address that the OpenTok servers will use to situate the session in the global
    /// OpenTok network. If you do not set a location hint, the OpenTok servers will be based
    /// on the first client connecting to the session.
    pub location: Option<&'a str>,
    /// Determines whether the session will transmit streams using the OpenTok Media Router
    /// ("routed") or not ("relayed"). By default, the setting is "relayed".
    /// With the media_mode parameter set to "relayed", the session will attempt to transmit
    /// streams directly between clients. If clients cannot connect due to firewall restrictions,
    /// the session uses the OpenTok TURN server to relay audio-video streams.
    pub media_mode: Option<MediaMode>,
    /// Whether the session is automatically archived ("always") or not ("manual").
    /// By default, the setting is "manual". To archive the session (either automatically or not),
    /// you must set the media_mode parameter to "routed".
    /// Archiving is currently unsupported.
    pub archive_mode: Option<ArchiveMode>,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct CreateSessionBody<'a> {
    archive_mode: String,
    location: Option<&'a str>,
    #[serde(rename = "p2p.preference")]
    p2p_preference: &'a str,
}

impl<'a> From<SessionOptions<'a>> for CreateSessionBody<'a> {
    fn from(options: SessionOptions) -> CreateSessionBody {
        CreateSessionBody {
            archive_mode: options
                .archive_mode
                .map(|mode| mode.to_string())
                .unwrap_or("manual".into()),
            location: options.location,
            p2p_preference: options
                .media_mode
                .map(|mode| {
                    if mode == MediaMode::Relayed {
                        "enabled"
                    } else {
                        "disabled"
                    }
                })
                .unwrap_or("disabled"),
        }
    }
}

#[derive(Deserialize)]
struct CreateSessionResponse {
    session_id: String,
}

#[derive(Debug)]
pub enum TokenRole {
    Publisher,
    Subscriber,
    Moderator,
}

impl fmt::Display for TokenRole {
    fn fmt(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        write!(formatter, "{}", format!("{:?}", self).to_lowercase())
    }
}

#[derive(Debug)]
struct TokenData<'a> {
    session_id: &'a str,
    create_time: u64,
    expire_time: u64,
    nonce: u64,
    role: TokenRole,
}

impl<'a> TokenData<'a> {
    pub fn new(session_id: &'a str, role: TokenRole) -> Self {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("Time went backwards, Doc!")
            .as_secs();
        let mut rng = rand::thread_rng();
        Self {
            session_id,
            create_time: now,
            expire_time: now + (60 * 60 * 24),
            nonce: rng.gen::<u64>(),
            role,
        }
    }
}

impl<'a> fmt::Display for TokenData<'a> {
    fn fmt(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        write!(
            formatter,
            "{}",
            format!(
                "session_id={}&create_time={}&expire_time={}&nonce={}&role={}",
                self.session_id, self.create_time, self.expire_time, self.nonce, self.role,
            )
        )
    }
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum VideoType {
    Camera,
    Screen,
    Custom,
}

impl fmt::Display for VideoType {
    fn fmt(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        write!(formatter, "{}", format!("{:?}", self).to_lowercase())
    }
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct StreamInfo {
    id: String,
    video_type: VideoType,
    name: String,
    layout_class_list: Vec<String>,
}

/// Top level entry point exposing the OpenTok server SDK functionality.
/// Contains methods for creating OpenTok sessions, generating tokens and
/// getting information about streams.
pub struct OpenTok {
    api_key: String,
    api_secret: String,
}

impl OpenTok {
    /// Create a new instance of OpenTok. Requires an OpenTok API key and
    /// the API secret for your TokBox account. Do not publicly share your
    /// API secret.
    pub fn new(api_key: String, api_secret: String) -> Self {
        Self {
            api_key,
            api_secret,
        }
    }

    /// Creates a new OpenTok session.
    /// On success, a session ID is provided.
    pub async fn create_session<'a>(
        &self,
        options: SessionOptions<'a>,
    ) -> Result<String, OpenTokError> {
        let body: CreateSessionBody = options.into();
        let endpoint = format!("{}{}", SERVER_URL, "/session/create");
        let mut response =
            http_client::post(&endpoint, &self.api_key, &self.api_secret, &body).await?;
        let response_str = response.body_string().await?;
        let mut response: Vec<CreateSessionResponse> =
            serde_json::from_str::<Vec<CreateSessionResponse>>(&response_str)
                .map_err(|_| OpenTokError::UnexpectedResponse(response_str.clone()))?;
        assert_eq!(response.len(), 1);
        match response.pop() {
            Some(session) => Ok(session.session_id),
            None => Err(OpenTokError::UnexpectedResponse(response_str)),
        }
    }

    pub fn generate_token(&self, session_id: &str, role: TokenRole) -> String {
        let token_data = TokenData::new(session_id, role);
        let signed = hmacsha1::hmac_sha1(
            self.api_secret.as_bytes(),
            token_data.to_string().as_bytes(),
        )
        .to_hex();
        let decoded = format!(
            "partner_id={}&sig={}:{}",
            self.api_key,
            signed,
            token_data.to_string()
        );
        let encoded = base64::encode(decoded);
        format!("T1=={}", encoded)
    }

    pub async fn get_stream_info(
        &self,
        session_id: &str,
        stream_id: &str,
    ) -> Result<StreamInfo, OpenTokError> {
        let endpoint = format!(
            "{}{}{}/session/{}/stream/{}",
            SERVER_URL, API_ENDPOINT_PATH_START, self.api_key, session_id, stream_id
        );
        let mut response = http_client::get(&endpoint, &self.api_key, &self.api_secret).await?;
        let response_str = response.body_string().await?;
        serde_json::from_str::<StreamInfo>(&response_str)
            .map_err(|_| OpenTokError::UnexpectedResponse(response_str.clone()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use futures::executor::LocalPool;
    use opentok_utils::common::Credentials;
    use opentok_utils::publisher::Publisher;
    use std::env;

    #[test]
    fn test_create_session_invalid_credentials() {
        let opentok = OpenTok::new("sancho".into(), "quijote".into());
        let mut pool = LocalPool::new();
        assert!(pool
            .run_until(opentok.create_session(SessionOptions::default()))
            .is_err());
    }

    #[test]
    fn test_create_session() {
        let api_key = env::var("OPENTOK_KEY").unwrap();
        let api_secret = env::var("OPENTOK_SECRET").unwrap();
        let opentok = OpenTok::new(api_key, api_secret);
        let mut pool = LocalPool::new();
        let session_id = pool
            .run_until(opentok.create_session(SessionOptions::default()))
            .unwrap();
        assert!(!session_id.is_empty());
    }

    #[test]
    fn test_generate_token() {
        let api_key = env::var("OPENTOK_KEY").unwrap();
        let api_secret = env::var("OPENTOK_SECRET").unwrap();
        let opentok = OpenTok::new(api_key, api_secret);
        let mut pool = LocalPool::new();
        let session_id = pool
            .run_until(opentok.create_session(SessionOptions::default()))
            .unwrap();
        assert!(!session_id.is_empty());
        let token = opentok.generate_token(&session_id, TokenRole::Publisher);
        assert!(!token.is_empty());
    }

    #[test]
    fn test_get_stream_info() {
        let api_key = env::var("OPENTOK_KEY").unwrap();
        let api_secret = env::var("OPENTOK_SECRET").unwrap();
        let opentok = OpenTok::new(api_key.clone(), api_secret);
        let mut pool = LocalPool::new();
        let session_id = pool
            .run_until(opentok.create_session(SessionOptions::default()))
            .unwrap();
        assert!(!session_id.is_empty());
        let token = opentok.generate_token(&session_id, TokenRole::Publisher);
        assert!(!token.is_empty());

        opentok::init().unwrap();

        let publisher = Publisher::new(
            Credentials {
                api_key,
                session_id: session_id.clone(),
                token,
            },
            Some(Box::new(move |publisher, stream_id| {
                let mut pool = LocalPool::new();
                let stream_info = pool
                    .run_until(opentok.get_stream_info(&session_id, &stream_id))
                    .unwrap();
                assert_eq!(stream_info.id, stream_id);
                publisher.stop();
            })),
        );

        publisher.run().unwrap();

        opentok::deinit().unwrap();
    }
}
