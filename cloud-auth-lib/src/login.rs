use super::{
    dto::login::*,
    dto::user_info::OHXAuthUser,
    CloudAuthError,
};
use chrono::{TimeZone, Duration as OldDuration};
use std::time::Duration;
use std::thread;
use std::collections::BTreeSet;

#[allow(unused_imports)]
use log::{debug, error, info, trace, warn};

pub fn refresh_token(mut session: UserSession) -> Result<UserSession, CloudAuthError> {
    let refresh_token = session.refresh_token.as_ref().ok_or(CloudAuthError::Generic("User session found, but no refresh token"))?;

    let token_request = TokenRequestForRefreshToken {
        refresh_token: refresh_token.clone(),
        client_id: session.client_id.clone(),
        grant_type: "refresh_token".to_string(),
    };

    let data = serde_urlencoded::to_string(token_request)?;

    let mut response = ureq::post("https://oauth.openhabx.com/token")
        .set("Content-Type", "application/x-www-form-urlencoded")
        .send_string(&data);
    if response.error() {
        return Err(CloudAuthError::HttpError("https://oauth.openhabx.com/token".into(), response.status_line().into()));
    }

    match response.status() {
        400 => {
            Err(CloudAuthError::GenericOwned(format!("Access token could not be refreshed. {}. Login required",
                                                     &serde_json::from_value::<ErrorResult>(response.into_json()?)?.error)))
        }
        200 => {
            let r: OAuthTokenResponse = serde_json::from_value(response.into_json()?)?;
            session.access_token = r.access_token;
            session.access_token_expires = chrono::Utc::now() + OldDuration::seconds(r.expires_in - 10);
            Ok(session)
        }
        v => {
            return Err(CloudAuthError::GenericOwned(format!("Unexpected response {} while refreshing access token: {}\nhttps://oauth.openhabx.com/token?auth={}",
                                                            v, &response.into_string()?, refresh_token)));
        }
    }
}

pub struct LoginDeviceFlow {
    pub client_id: String,
    pub device_code: String,
    pub verification_uri: String,
    expires_in: i64,
    pub expires_time: i64,

    pub access_token: Option<String>,
    pub refresh_token: Option<String>,
    pub scopes: BTreeSet<String>,
}

impl LoginDeviceFlow {
    pub fn new(token_request: AuthRequest) -> Result<LoginDeviceFlow, CloudAuthError> {
        let data = serde_urlencoded::to_string(token_request).expect("AuthRequest encoding");
        let response = ureq::post("https://oauth.openhabx.com/authorize")
            .set("Content-Type", "application/x-www-form-urlencoded")
            .send_string(&data);
        if response.error() {
            return Err(CloudAuthError::HttpError("https://oauth.openhabx.com/authorize".into(), response.status_line().into()));
        }
        if response.status() != 200 {
            let message = match response.status() {
                400 => serde_json::from_value::<ErrorResult>(response.into_json()?).expect("An error json result").error,
                _ => response.into_string()?
            };
            return Err(CloudAuthError::GenericOwned(format!("Could not start authorisation process: {}", &message)));
        }

        let device_flow_response: DeviceFlowResponse = serde_json::from_value(response.into_json()?)?;

        Ok(LoginDeviceFlow {
            client_id: token_request.client_id,
            device_code: device_flow_response.device_code,
            verification_uri: device_flow_response.verification_uri,
            expires_in: device_flow_response.expires_in,
            expires_time: device_flow_response.expires_in + chrono::Utc::now().timestamp(),
            access_token: None,
            refresh_token: None,
            scopes: BTreeSet::new(),
        })
    }

    pub fn diff(&self) -> i64 {
        self.expires_time - chrono::Utc::now().timestamp()
    }

    pub fn wait_for_user_blocking(&mut self, check_every: Duration) -> Result<(), CloudAuthError> {
        loop {
            thread::sleep(check_every);

            let token_request = TokenRequestForDevice {
                device_code: self.device_code.clone(),
                client_id: self.client_id.clone(),
                grant_type: "urn:ietf:params:oauth:grant-type:device_code".to_string(),
            };
            let data = serde_urlencoded::to_string(token_request).expect("AuthRequest encoding");
            let response = ureq::post("https://oauth.openhabx.com/token")
                .set("Content-Type", "application/x-www-form-urlencoded")
                .send_string(&data);
            if response.error() {
                return Err(CloudAuthError::HttpError("https://oauth.openhabx.com/token".into(), response.status_line().into()));
            }

            match response.status() {
                200 => {
                    let r: OAuthTokenResponse = serde_json::from_value(response.into_json()?).unwrap();
                    self.access_token = Some(r.access_token);
                    self.refresh_token = r.refresh_token;
                    self.scopes = r.scope;
                    self.expires_in = r.expires_in;
                    self.expires_time = r.expires_in + chrono::Utc::now().timestamp();
                    return Ok(());
                }
                400 => {
                    let response = serde_json::from_value::<ErrorResult>(response.into_json()?)?;
                    if &response.error != "authorization_pending" {
                        return Err(CloudAuthError::GenericOwned(format!("Server response: {}", &response.error)));
                    }
                }
                _ => {
                    return Err(CloudAuthError::GenericOwned(format!("Server response: {}", response.into_string()?)));
                }
            };
            if self.diff() <= 0 {
                return Err(CloudAuthError::Timeout);
            }
        }
    }

    /// Get user information if possible and return a user session with valid access token and
    /// depending on the requested scope also a refresh token.
    pub fn session(self) -> Result<UserSession, CloudAuthError> {
        let access_token = self.access_token.ok_or(CloudAuthError::Generic("Invalid access token"))?;

        let response = ureq::get("https://oauth.openhabx.com/userinfo")
            .auth_kind("Bearer", &access_token)
            .call();

        if response.error() {
            return Err(CloudAuthError::HttpError("https://oauth.openhabx.com/userinfo".into(), response.status_line().into()));
        }

        let user_data: OHXAuthUser = serde_json::from_value(response.into_json()?)?;

        Ok(UserSession {
            refresh_token: self.refresh_token,
            access_token,
            access_token_expires: chrono::Utc.timestamp(self.expires_time, 0),
            client_id: self.client_id,
            user_id: user_data.localId.unwrap_or_default(),
            user_email: user_data.email.unwrap_or_default(),
            user_display_name: user_data.displayName.unwrap_or_default(),
        })
    }
}

