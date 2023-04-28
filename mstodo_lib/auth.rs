//! Only [[DeviceCodeAuthentication]] is implemented.
//! It takes two steps to authenticate:
//!     1. Get a device code from the server and wait
//!         for user to enter the code on the website.
//!     2. Get an access token from the server.
//! The access token will be saved in the credential store provided by the OS
//! when available. otherwise it will be saved in a file under the user's home

const CLIENT_ID: &'static str = "c85cbdd1-4823-4bc8-b02e-2f3f7caa9dd7";
const API_SCOPE: &'static str = "offline_access User.Read Tasks.ReadWrite";
const DEVICE_CODE_ENDPOINT: &str =
    "https://login.microsoftonline.com/e620629d-ca12-4421-8f81-ba47552f618d/oauth2/v2.0/devicecode";
const AUTH_ENDPOINT: &str =
    "https://login.microsoftonline.com/e620629d-ca12-4421-8f81-ba47552f618d/oauth2/v2.0/token";
/// Authentication requests
mod requests {
    use super::{responses::DeviceCodeAuthenticationResponse, CLIENT_ID};

    /// Request to get a device code from the server
    /// The device code will be used to get an access token
    /// See "https://docs.microsoft.com/en-us/azure/active-directory/develop/v2-oauth2-device-code"
    /// for more information
    #[derive(serde::Serialize, Debug, Clone, PartialEq, PartialOrd)]
    pub(super) struct DeviceCodeAuthenticationRequest<'req> {
        pub client_id: &'req str,

        pub scope: &'req str,
    }

    /// Request to get an access token from the server
    /// The access token will be used to access the API
    /// See "https://docs.microsoft.com/en-us/azure/active-directory/develop/v2-oauth2-auth-code-flow"
    /// for more information
    #[derive(serde::Serialize, Debug, Clone, PartialEq, PartialOrd)]
    pub(super) struct AuthenticationRequest<'req> {
        pub client_id: &'req str,
        #[serde(rename = "code")]
        pub device_code: &'req str,
        pub grant_type: &'req str,
    }

    impl<'req> From<&'req DeviceCodeAuthenticationResponse> for AuthenticationRequest<'req> {
        fn from(resp: &'req DeviceCodeAuthenticationResponse) -> Self {
            Self {
                client_id: CLIENT_ID,
                device_code: &resp.device_code,
                grant_type: "urn:ietf:params:oauth:grant-type:device_code",
            }
        }
    }
}

pub mod responses {
    /// Response from the server when requesting a device code
    /// The user will need to enter the user code on the website
    /// and wait for the device code to be authorized.
    #[derive(serde::Deserialize, Debug, Clone, PartialEq, PartialOrd)]
    pub(super) struct DeviceCodeAuthenticationResponse {
        pub device_code: String,
        pub user_code: String,
        pub verification_uri: String,
        pub expires_in: u64,
        pub interval: u64,
        pub message: String,
    }

    /// Error types when requesting an access token
    /// See "https://docs.microsoft.com/en-us/azure/active-directory/develop/v2-oauth2-auth-code-flow"
    /// for more information
    #[derive(serde::Deserialize, Debug, Clone, PartialEq)]
    #[serde(rename_all = "snake_case")]
    pub enum AuthorizationError {
        AuthorizationPending,
        AuthorizationDeclined,
        BadVerificationCode,
        ExpiredToken,
    }

    /// Error response from the server when requesting an access token
    /// See "https://docs.microsoft.com/en-us/azure/active-directory/develop/v2-oauth2-auth-code-flow"
    /// for more information
    #[derive(serde::Deserialize, Debug, Clone, PartialEq)]
    pub struct DeviceCodeAhenticationError {
        pub error: AuthorizationError,
        pub error_description: String,
        pub error_codes: Vec<u64>,
        pub timestamp: String,
        pub trace_id: String,
        pub correlation_id: String,
    }

    /// Response from the server when requesting an access token
    /// See "https://docs.microsoft.com/en-us/azure/active-directory/develop/v2-oauth2-auth-code-flow"
    /// for more information
    #[derive(serde::Deserialize, Debug, Clone, PartialEq)]
    pub struct AuthenticationResponse {
        pub token_type: String,
        pub scope: String,
        pub expires_in: u64,
        pub ext_expires_in: u64,
        pub access_token: String,
        pub refresh_token: String,
        pub id_token: Option<String>,
    }
}
use std::time::Duration;

use responses::*;
pub struct DeviceCodeAuthentication {
    http_client: reqwest::Client,
}

impl Default for DeviceCodeAuthentication {
    fn default() -> Self {
        Self {
            http_client: reqwest::Client::new(),
        }
    }
}

impl DeviceCodeAuthentication {
    pub fn new() -> Self {
        Self::default()
    }
    async fn authenticate_with_device_code(
        &self,
    ) -> Result<AuthenticationResponse, super::error::AuthenticationError> {
        let req_body = requests::DeviceCodeAuthenticationRequest {
            client_id: CLIENT_ID.as_ref(),
            scope: API_SCOPE.as_ref(),
        };
        println!("Device Code REquest: {:?}", req_body);
        let resp_raw = self
            .http_client
            .post(DEVICE_CODE_ENDPOINT)
            .form(&req_body)
            .send()
            .await?;
        if !resp_raw.status().is_success() {
            return Err(super::error::AuthenticationError::UnexpectedResponse(
                resp_raw.text().await.unwrap(),
            ));
        }
        let resp = resp_raw.json::<DeviceCodeAuthenticationResponse>().await?;
        let poll_interval = Duration::from_secs(resp.interval);
        println!("{}", resp.message);

        // polling for authentication status as instructed by the server
        let poll_req = requests::AuthenticationRequest::from(&resp);
        loop {
            let poll_resp_raw = self
                .http_client
                .post(AUTH_ENDPOINT)
                .form(&poll_req)
                .send()
                .await?;

            let status = poll_resp_raw.status();

            // user has authorized the device code
            if status.is_success() {
                let res = poll_resp_raw.json().await?;
                break Ok(res);
            }
            let poll_err = poll_resp_raw
                .json::<responses::DeviceCodeAhenticationError>()
                .await
                .map_err(|e| {
                    crate::error::AuthenticationError::UnexpectedResponse(e.to_string())
                })?;
            if poll_err.error != AuthorizationError::AuthorizationPending {
                break Err(crate::error::AuthenticationError::AuthenticationFailed);
            }
            tokio::time::sleep(poll_interval).await;
        }
    }

    fn authenticate_with_refresh_token(&self) -> Result<(), super::error::AuthenticationError> {
         let token = keyring::CredentialBuilder::build(&self, target, service, user)
    }
}
