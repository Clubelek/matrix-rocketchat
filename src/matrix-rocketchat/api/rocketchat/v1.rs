use reqwest::header::Headers;
use reqwest::{Method, StatusCode};
use serde_json;
use slog::Logger;

use api::RestApi;
use errors::*;
use i18n::*;
use super::Endpoint;

/// Login endpoint path
pub const LOGIN_PATH: &'static str = "/api/v1/login";

/// V1 login endpoint
pub struct LoginEndpoint<'a> {
    /// Base URL to call the endpoint.
    base_url: String,
    payload: LoginPayload<'a>,
}

/// Payload of the login endpoint
#[derive(Serialize)]
pub struct LoginPayload<'a> {
    username: &'a str,
    password: &'a str,
}

impl<'a> Endpoint for LoginEndpoint<'a> {
    fn method(&self) -> Method {
        Method::Post
    }

    fn url(&self) -> String {
        self.base_url.clone() + LOGIN_PATH
    }

    fn payload(&self) -> Result<String> {
        let payload = serde_json::to_string(&self.payload).chain_err(|| ErrorKind::InvalidJSON("Could not serialize login payload".to_string()))?;
        Ok(payload)
    }

    fn headers(&self) -> Option<Headers> {
        None
    }
}

#[derive(Deserialize)]
/// Response payload from the Rocket.Chat login endpoint.
pub struct LoginResponse {
    /// Status of the response (success, error)
    pub status: String,
    /// Data of the response
    pub data: Credentials,
}

/// User credentials.
#[derive(Deserialize)]
pub struct Credentials {
    /// The authentication token for Rocket.Chat
    #[serde(rename = "authToken")]
    pub auth_token: String,
    /// The users unique id on the rocketchat server.
    #[serde(rename = "userId")]
    pub user_id: String,
}

#[derive(Clone)]
/// Rocket.Chat REST API v1
pub struct RocketchatApi {
    /// URL to call the API
    pub base_url: String,
    /// Access token for authentication
    pub access_token: Option<String>,
    /// Logger passed to the Rocketchat API
    logger: Logger,
}

impl RocketchatApi {
    /// Create a new `RocketchatApi`.
    pub fn new(base_url: String, access_token: Option<String>, logger: Logger) -> RocketchatApi {
        RocketchatApi {
            base_url: base_url,
            access_token: access_token,
            logger: logger,
        }
    }
}

impl super::RocketchatApi for RocketchatApi {
    fn login(&self, username: &str, password: &str) -> Result<(String, String)> {
        let login_endpoint = LoginEndpoint {
            base_url: self.base_url.clone(),
            payload: LoginPayload {
                username: username,
                password: password,
            },
        };

        let (body, status_code) = RestApi::call_rocketchat(&login_endpoint)?;
        if !status_code.is_success() {
            return Err(build_error(login_endpoint.url(), &body, &status_code));
        }

        let login_response: LoginResponse = serde_json::from_str(&body).chain_err(|| {
                ErrorKind::InvalidJSON(format!("Could not deserialize response from Rocket.Chat login API endpoint: `{}`",
                                               body))
            })?;
        Ok((login_response.data.user_id, login_response.data.auth_token))
    }
}

fn build_error(endpoint: String, body: &str, status_code: &StatusCode) -> Error {
    let json_error_msg = format!("Could not deserialize error from Rocket.Chat API endpoint {} with status code {}: `{}`",
                                 endpoint,
                                 status_code,
                                 body);
    let json_error = ErrorKind::InvalidJSON(json_error_msg);
    let rocketchat_error_resp: RocketchatErrorResponse =
        match serde_json::from_str(body).chain_err(|| json_error).map_err(Error::from) {
            Ok(rocketchat_error_resp) => rocketchat_error_resp,
            Err(err) => {
                return err;
            }
        };

    if *status_code == StatusCode::Unauthorized {
        return Error {
            error_chain: ErrorKind::AuthenticationFailed(rocketchat_error_resp.message).into(),
            user_message: Some(t!(["errors", "authentication_failed"])),
        };
    }

    Error::from(ErrorKind::RocketchatError(rocketchat_error_resp.message))
}