use reqwest::header::{HeaderMap, HeaderValue};
use serde::Deserialize;
use serde_json::json;

use ohx_addon_publish::addons::*;

pub struct GithubClient(pub reqwest::Client);

#[derive(Deserialize)]
pub struct GithubGraphQLResponse {
    data: GithubRepositoryResponse,
}

#[derive(Deserialize)]
pub struct GithubRepositoryResponse {
    repository: GithubObjectResponse,
}

#[derive(Deserialize)]
pub struct GithubObjectResponse {
    object: GithubShaResponse,
}

#[derive(Deserialize)]
pub struct GithubShaResponse {
    oid: Option<String>,
    text: Option<String>,
}

#[derive(Deserialize)]
pub struct GithubGraphQLStatsResponse {
    data: GithubRepositoryStatsResponse,
}

#[derive(Deserialize)]
pub struct GithubRepositoryStatsResponse {
    repository: GithubStatsResponse,
}

#[derive(Deserialize)]
pub struct GithubStatsResponse {
    pub(crate) issues: GithubTotalCountResponse,
    pub(crate) stargazers: GithubTotalCountResponse,
}

#[allow(non_snake_case)]
#[derive(Deserialize)]
pub struct GithubTotalCountResponse {
    pub(crate) totalCount: u64
}


pub fn put_metadata_file(client: &GithubClient, sha: &str, content: &AddonMapStats, reason: &str) -> Result<(), failure::Error> {
    let url =
        "https://api.github.com/repos/openhab-nodes/addons-registry/contents/extensions_stats.json";

    let content = serde_json::to_string_pretty(&content)?;
    let content = base64::encode(&content);
    client.0.put(url).json(&json!({
        "message": reason,
        "content": content,
        "sha": sha
    })).send()?;
    Ok(())
}

pub fn put_data_file(client: &GithubClient, sha: &str, content: &AddonEntryMap, reason: &str) -> Result<(), failure::Error> {
    let url = "https://api.github.com/repos/openhab-nodes/addons-registry/contents/extensions.json";
    let content = serde_json::to_string_pretty(&content)?;
    let content = base64::encode(&content);
    client.0.put(url).json(&json!({
        "message": reason,
        "content": content,
        "sha": sha
    })).send()?;
    Ok(())
}

/// Return a tuple (file_content, sha)
pub fn get_data_detail_file(client: &GithubClient, addon_id: &str) -> Result<(AddonDetailedInfo, Option<String>), failure::Error> {
    let query = format!("query {{repository(owner: \"openhab-nodes\", name: \"addons-registry\") {{ object(expression: \"master:{}.json\") {{ ... on Blob {{text, oid}}}} }} }}", addon_id);
    let t = client.0.post("https://api.github.com/graphql").json(&json!({
        "query": &query
    })).send()?.text()?;
    //println!("DATA {}", &t);
    let r: GithubGraphQLResponse = serde_json::from_str(&t)?;
    Ok((serde_json::from_str(&r.data.repository.object.text.unwrap().replace(r#"\""#, "\""))?, r.data.repository.object.oid))
}

pub fn put_data_detail_file(client: &GithubClient, addon_id: &str, sha: Option<String>, content: &AddonDetailedInfo, reason: &str) -> Result<(), failure::Error> {
    let url = format!("https://api.github.com/repos/openhab-nodes/addons-registry/contents/{}.json", addon_id);
    let content = serde_json::to_string_pretty(&content)?;
    let content = base64::encode(&content);
    client.0.put(&url).json(&json!({
        "message": reason,
        "content": content,
        "sha": sha
    })).send()?;
    Ok(())
}

pub fn get_metadata_sha(client: &GithubClient) -> Result<String, failure::Error> {
    let get_sha = r#"{"query": "query {repository(owner: \"openhab-nodes\", name: \"addons-registry\") { object(expression: \"master:extensions_stats.json\") { ... on Blob {oid}}}}""#;
    let t = client.0.post("https://api.github.com/graphql").body(get_sha).send()?.text()?;
    //println!("DATA {}", &t);
    let r: GithubGraphQLResponse = serde_json::from_str(&t)?;
    Ok(r.data.repository.object.oid.unwrap())
}

/// Return a tuple (file_content, sha)
pub fn get_metadata_content(client: &GithubClient) -> Result<(AddonMapStats, String), failure::Error> {
    let get_sha = r#"{"query": "query {repository(owner: \"openhab-nodes\", name: \"addons-registry\") { object(expression: \"master:extensions_stats.json\") { ... on Blob {text, oid}}}}""#;
    let t = client.0.post("https://api.github.com/graphql").body(get_sha).send()?.text()?;
    //println!("DATA {}", &t);
    let r: GithubGraphQLResponse = serde_json::from_str(&t)?;
    Ok((serde_json::from_str(&r.data.repository.object.text.unwrap().replace(r#"\""#, "\""))?, r.data.repository.object.oid.unwrap()))
}

/// Return a tuple (file_content, sha)
pub fn get_data_content(client: &GithubClient) -> Result<(AddonEntryMap, String), failure::Error> {
    let get_sha = r#"{"query": "query {repository(owner: \"openhab-nodes\", name: \"addons-registry\") { object(expression: \"master:extensions.json\") { ... on Blob {text, oid}}}}""#;
    let t = client.0.post("https://api.github.com/graphql").body(get_sha).send()?.text()?;
    //println!("DATA {}", &t);
    let r: GithubGraphQLResponse = serde_json::from_str(&t)?;
    Ok((serde_json::from_str(&r.data.repository.object.text.unwrap().replace(r#"\""#, "\""))?, r.data.repository.object.oid.unwrap()))
}

pub fn get_data_sha(client: &GithubClient) -> Result<String, failure::Error> {
    let get_sha = r#"{"query": "query {repository(owner: \"openhab-nodes\", name: \"addons-registry\") { object(expression: \"master:extensions.json\") { ... on Blob {oid}}}}""#;
    let t = client.0.post("https://api.github.com/graphql").body(get_sha).send()?.text()?;
    //println!("DATA {}", &t);
    let r: GithubGraphQLResponse = serde_json::from_str(&t)?;
    Ok(r.data.repository.object.oid.unwrap())
}


/// Return a tuple (file_content, sha)
pub fn get_stats_for_repo(client: &GithubClient, owner: &str, name: &str) -> Result<GithubStatsResponse, failure::Error> {
    let query = format!("query {{repository(owner: \"{}\", name: \"{}\") {{issues {{totalCount}}, stargazers {{totalCount}}}}}}", owner, name);
    let t = client.0.post("https://api.github.com/graphql").json(&json!({
        "query": &query
    })).send()?.text()?;
    //println!("DATA {}", &t);
    let r: GithubGraphQLStatsResponse = serde_json::from_str(&t)?;
    Ok(r.data.repository)
}

#[derive(Deserialize)]
struct GithubCredentials {
    user: String,
    password: String,
}

pub fn create_client(github_credentials: &str) -> Result<GithubClient, failure::Error> {
    let g: GithubCredentials = serde_json::from_str(github_credentials)) ? ;
    let g = base64::encode_config(&format!("{}:{}", g.user, g.password), base64::STANDARD);
    let mut headers: HeaderMap<HeaderValue> = HeaderMap::with_capacity(2);
    headers.insert("authorization", format!("Basic {}", g).parse().unwrap());
    headers.insert("user-agent", "OpenhabX-Nodes-Bot".parse().unwrap());
    let g = GithubClient(
        reqwest::Client::builder()
            .connect_timeout(Some(std::time::Duration::from_secs(2)))
            .timeout(Some(std::time::Duration::from_secs(5)))
            .default_headers(headers)
            .build()?,
    );
    return Ok(g);
}

#[test]
fn get_stats_for_repo_test() {
    let client = create_client().unwrap();
    let data = get_stats_for_repo(&client, "openhab-nodes", "addons-registry").unwrap();
    assert_eq!(data.issues.totalCount, 0);
    assert_eq!(data.stargazers.totalCount, 0);
}

#[test]
fn get_metadata_file_test() {
    let client = create_client().unwrap();

    let (mut data, sha) = get_metadata_content(&client).unwrap();
    let cmp: i64;
    match data.get_mut("demo-entry") {
        Some(data) => {
            data.d += 1;
            cmp = data.d;
            println!("UDATATE D {}", cmp);
        }
        None => {
            data.insert("demo-entry".into(), AddonStats {
                v: 1,
                p: 5,
                d: 12,
                s: 1000,
                iss: 7,
                t: 121233556756,
            });
            cmp = 12;
        }
    };

    put_metadata_file(&client, &sha, &data, "CI Test commit").unwrap();

    let data: AddonMapStats = get_metadata_content(&client).unwrap().0;
    assert!(data.contains_key("demo-entry"));
    let data = data.get("demo-entry").unwrap();
    assert_eq!(data.d, cmp);
}
