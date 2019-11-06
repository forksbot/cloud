use serde::Deserialize;
use serde_json::json;
use reqwest::header::{AUTHORIZATION, HeaderName, HeaderMap, USER_AGENT};
use chrono::Duration;

use crate::credentials::Credentials;
use crate::jwt::create_jwt_encoded;

#[derive(Deserialize)]
pub(crate) struct TravisEnvVarsEntry {
    id: String,
    name: String,
}

#[derive(Deserialize)]
pub(crate) struct TravisResponse {
    env_vars: Vec<TravisEnvVarsEntry>,
}

pub(crate) fn set_env_var(token: &str, repository_names: Vec<String>, credentials: &Credentials) -> Result<String, failure::Error> {
    ////// Create reqwest client with auth header etc
    let mut headers = HeaderMap::new();
    headers.insert(AUTHORIZATION, format!("token {}", token).parse()?);
    headers.insert(HeaderName::from_static("travis-api-version"), "3".parse()?);
    headers.insert(USER_AGENT, "OHX 1.0".parse()?);

    let client = reqwest::ClientBuilder::new()
        .connect_timeout(std::time::Duration::from_secs(3))
        .default_headers(headers)
        .build()?;

    // Create access token and json body for the http patch or post call
    let jwts = create_jwt_encoded(&credentials, Some(["addons"].iter()), Duration::hours(6),Some(credentials.client_id.clone()))?;
    let body = json!({
        "env_var.name": "DEPLOY_ACCESS_TOKEN",
        "env_var.value": jwts,
        "env_var.public": false
        });

    // Perform an update call for each repository listed in repositories.json
    let mut response = String::new();
    for repository_name in repository_names {
        let travis_url = format!("https://api.travis-ci.org/repo/{}%2F{}/env_vars", "openhab-nodes", repository_name);
        let travis_response = client.get(&travis_url).send()?.text()?;

        let travis_response_json: Result<TravisResponse, _> = serde_json::from_str(&travis_response);
        match travis_response_json {
            Ok(r) => {
                // It is either a create environment variable or patch existing variable call
                let env_var = r.env_vars.iter().find(|f| f.name == "DEPLOY_ACCESS_TOKEN");
                match env_var {
                    Some(entry) => {
                        let travis_url = format!("https://api.travis-ci.org/repo/{}%2F{}/env_var/{}", "openhab-nodes", repository_name, &entry.id);
                        client.patch(&travis_url).json(&body).send()?;
                        response += &format!("Updated var {}\n", &repository_name);
                    }
                    None => {
                        let travis_url = format!("https://api.travis-ci.org/repo/{}%2F{}/env_vars", "openhab-nodes", repository_name);
                        client.post(&travis_url).json(&body).send()?;
                        response += &format!("Created var {}\n", &repository_name);
                    }
                }
            }
            Err(e) => {
                response += &format!("Failed on {}: {:?}. Got: {}\n", &repository_name, e, &travis_response);
            }
        }
    }
    Ok(response)
}