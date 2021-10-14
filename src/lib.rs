use serde::{Deserialize, Serialize};
use std::fmt;
use thiserror::Error;

mod http_client;

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
    location: Option<&'a str>,
    /// Determines whether the session will transmit streams using the OpenTok Media Router
    /// ("routed") or not ("relayed"). By default, the setting is "relayed".
    /// With the media_mode parameter set to "relayed", the session will attempt to transmit
    /// streams directly between clients. If clients cannot connect due to firewall restrictions,
    /// the session uses the OpenTok TURN server to relay audio-video streams.
    media_mode: Option<MediaMode>,
    /// Whether the session is automatically archived ("always") or not ("manual").
    /// By default, the setting is "manual". To archive the session (either automatically or not),
    /// you must set the media_mode parameter to "routed".
    /// Archiving is currently unsupported.
    archive_mode: Option<ArchiveMode>,
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
        let mut response =
            http_client::post("/session/create", &self.api_key, &self.api_secret, &body).await?;
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
}

#[cfg(test)]
mod tests {
    use super::*;

    use futures::executor::LocalPool;
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
}
