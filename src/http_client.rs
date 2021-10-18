use crate::OpenTokError;

use jsonwebtoken::{encode, EncodingKey, Header};
use rand::Rng;
use serde::Serialize;
use std::time::{SystemTime, UNIX_EPOCH};

static AUTH_HEADER: &str = "X-OPENTOK-AUTH";
static ACCEPT: &str = "Accept";
static JSON: &str = "application/json";

#[derive(Debug, Serialize)]
struct Claims<'a> {
    iss: &'a str,
    ist: &'static str,
    iat: u64,
    exp: u64,
    jti: u64,
}

impl<'a> Claims<'a> {
    fn new(api_key: &'a str) -> Self {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("Time went backwards, Doc!")
            .as_secs();
        let mut rng = rand::thread_rng();
        Self {
            iss: api_key,
            ist: "project",
            iat: now,
            exp: now + (3 * 60),
            jti: rng.gen::<u64>(),
        }
    }
}

fn auth_header(api_key: &str, api_secret: &str) -> Result<String, OpenTokError> {
    let claims = Claims::new(api_key);
    encode(
        &Header::default(),
        &claims,
        &EncodingKey::from_secret(api_secret.as_ref()),
    )
    .map_err(|_| OpenTokError::EncodingError)
}

async fn from_surf_response(response: surf::Result) -> Result<surf::Response, OpenTokError> {
    match response {
        Ok(mut response) => match response.status().into() {
            200..=299 => Ok(response),
            _ => {
                let body = response
                    .body_string()
                    .await
                    .map_err(|_| OpenTokError::UnexpectedResponse(format!("{:?}", response)))?;
                let error = surf::Error::from_str(response.status(), body);
                Err(error.into())
            }
        },
        Err(error) => Err(error.into()),
    }
}

pub async fn post(
    endpoint: &str,
    api_key: &str,
    api_secret: &str,
    body: &impl Serialize,
) -> Result<surf::Response, OpenTokError> {
    let auth_header = auth_header(api_key, api_secret)?;
    let mut req = surf::post(endpoint).build();
    req.set_header(AUTH_HEADER, &auth_header);
    req.set_header(ACCEPT, JSON);
    req.body_form(body)
        .map_err(|_| OpenTokError::EncodingError)?;
    from_surf_response(surf::client().send(req).await).await
}

pub async fn get(
    endpoint: &str,
    api_key: &str,
    api_secret: &str,
) -> Result<surf::Response, OpenTokError> {
    let auth_header = auth_header(api_key, api_secret)?;
    let mut req = surf::get(endpoint).build();
    req.set_header(AUTH_HEADER, &auth_header);
    req.set_header(ACCEPT, JSON);
    from_surf_response(surf::client().send(req).await).await
}
