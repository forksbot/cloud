// own
use crate::dto::{db};
use crate::responder_type::MyResponder;
use crate::github;
use ohx_addon_publish::addons;

// External, controlled libraries
use cloud_vault::{
    guard_oauth_jwt_access, guard_rate_limiter::RateLimiter,
};
use firestore_db_and_auth::{
    documents, sessions::service_account::Session as SASession,
};

#[allow(unused_imports)]
use log::{debug, error, info, trace, warn};

// External libraries
use rocket::{delete, get, post};

// std
use rocket_contrib::json::Json;
use std::ops::Deref;
use std::sync::Mutex;
use std::collections::HashMap;
use ohx_addon_publish::addons::{AddonRegistryEntry, AddonDetailedInfo};

const CREDENTIALS_GOOGLE_SERVICE_ACCOUNT_INDEX: usize = 0;
const CREDENTIALS_OHX_SERVICE_ACCOUNT_INDEX: usize = 1;

/// Empty default route
#[get("/")]
pub fn index() -> &'static str {
    ""
}


pub struct RatingAccumulator {
    pub points: i64,
    pub voters: u64,
    pub downloads: i64,
}

fn accumulate_ratings(session: &SASession, documents_to_remove: &mut Vec<String>, addon_stats_to_update: &mut HashMap<String, RatingAccumulator>) -> Result<(), MyResponder> {
    let list: documents::List<db::RatingsInDB, _> = documents::list(session, "ratings");
    for metadata_and_doc in list {
        let (doc, metadata) = metadata_and_doc?;
        // let name = documents::abs_to_rel(metadata.name.as_ref().unwrap());
        info!("Found rating {} - {}", doc.addon_id, doc.rate);

        match addon_stats_to_update.get_mut(&doc.addon_id) {
            Some(val) => {
                val.points += doc.rate - doc.last_rating;
                val.voters += match doc.last_rating {
                    0 => 1, // if this is the first rating, increase the voters counter
                    _ => 0
                }
            }
            None => {
                addon_stats_to_update.insert(doc.addon_id, RatingAccumulator {
                    points: doc.rate - doc.last_rating,
                    voters: match doc.last_rating {
                        0 => 1, // if this is the first rating, increase the voters counter
                        _ => 0
                    },
                    downloads: 0,
                });
            }
        };

        documents_to_remove.push(metadata.name);
    }
    Ok(())
}


fn accumulate_downloads(session: &SASession, documents_to_remove: &mut Vec<String>, addon_stats_to_update: &mut HashMap<String, RatingAccumulator>) -> Result<(), MyResponder> {
    let list: documents::List<db::DownloadsInDB, _> = documents::list(session, "downloads");
    for metadata_and_doc in list {
        let (doc, metadata) = metadata_and_doc?;
        // let name = documents::abs_to_rel(metadata.name.as_ref().unwrap());
        info!("Found download entry {} - {}", doc.addon_id, doc.installed);

        match addon_stats_to_update.get_mut(&doc.addon_id) {
            Some(val) => {
                val.downloads += doc.installed;
            }
            None => {
                addon_stats_to_update.insert(doc.addon_id, RatingAccumulator {
                    points: 0,
                    voters: 0,
                    downloads: doc.installed,
                });
            }
        };

        documents_to_remove.push(metadata.name);
    }
    Ok(())
}

fn update_stats_for_repo(stats: &mut addons::AddonMapStats, github_client: &github::GithubClient, addon_id: &str, github: &str, timestamp: i64) -> bool {
    let parts: Vec<&str> = github.split("/").collect();
    let position = parts.iter().position(|f| f.contains("github.com"));
    let owner = position.and_then(|f| parts.get(f + 1));
    let repo_name = position.and_then(|f| parts.get(f + 2));
    if owner.is_none() || repo_name.is_none() {
        warn!("Unexpected Github link for '{}': {}", addon_id, github);
        return false;
    }

    let needs_update = match stats.get(addon_id) {
        Some(v) => v.t != timestamp,
        None => true
    };

    if !needs_update {
        return false;
    }

    match github::get_stats_for_repo(&github_client, owner.unwrap(), repo_name.unwrap()) {
        Ok(repo_stats) => {
            match stats.get_mut(addon_id) {
                Some(v) => {
                    v.s = repo_stats.stargazers.totalCount;
                    v.iss = repo_stats.issues.totalCount;
                    v.t = timestamp;
                }
                None => {
                    stats.insert(addon_id.to_owned(), addons::AddonStats {
                        v: 0,
                        p: 0,
                        d: 0,
                        s: repo_stats.stargazers.totalCount,
                        iss: repo_stats.issues.totalCount,
                        t: timestamp,
                    });
                }
            }
        }
        Err(e) => warn!("Error while fetching updates for {} - {}", &github, e)
    };
    return true;
}

#[get("/update_stats")]
pub fn update_stats(
    oauth_user: guard_oauth_jwt_access::OAuthIdentity,
    firebase: rocket::State<Mutex<SASession>>,
    github_client: rocket::State<github::GithubClient>,
) -> Result<String, MyResponder> {
    // Only the google account is allowed to call this endpoint
    if oauth_user.credentials_index != CREDENTIALS_GOOGLE_SERVICE_ACCOUNT_INDEX {
        return Err(MyResponder::access_denied("ONLY_SERVICE_ACCOUNT",
                                              "Only the google CI account is allowed to call this endpoint",
        ));
    }

    let timestamp = chrono::Utc::now().timestamp_millis();

    let session_mutex = firebase.lock()?;
    let session: &SASession = session_mutex.deref();

    let mut documents_to_remove: Vec<String> = Vec::new();
    let mut addon_stats_to_update: HashMap<String, RatingAccumulator> = HashMap::new();

    // Fetch ratings and accumulate into documents_to_remove and addon_stats_to_update
    accumulate_ratings(session, &mut documents_to_remove, &mut addon_stats_to_update)?;

    // Fetch downloads and accumulate into documents_to_remove and addon_stats_to_update
    accumulate_downloads(session, &mut documents_to_remove, &mut addon_stats_to_update)?;

    let (mut stats, sha) = github::get_metadata_content(&github_client)?;

    let mut has_changed = false;

    // Apply ratings
    if addon_stats_to_update.len() > 0 {
        has_changed = true;
        for (addon_id, ratings) in addon_stats_to_update {
            match stats.get_mut(&addon_id) {
                Some(v) => {
                    v.v += ratings.voters;
                    v.p += ratings.points;
                    v.d += ratings.downloads;
                    v.t = timestamp;
                }
                None => {
                    stats.insert(addon_id, addons::AddonStats {
                        v: ratings.voters,
                        p: ratings.points,
                        d: ratings.downloads,
                        s: 0,
                        iss: 0,
                        t: timestamp,
                    });
                }
            }
        }
    }

    // Update github stars and issue count
    match addons::get_addons_registry(&github_client.inner().0) {
        Ok(addons) => {
            for (addon_id, addon) in addons {
                // Expecting a github link: https://github.com/openhab/openhab2-addons
                if let Some(github) = addon.entry.github {
                    has_changed |= update_stats_for_repo(&mut stats, &github_client, &addon_id, &github, timestamp);
                }
            }
        }
        Err(e) => warn!("Error while getting the addon repo file {}", e),
    };

    if has_changed {
        github::put_metadata_file(&github_client, &sha, &stats, "Statistics update")?;
    }

    for path in documents_to_remove {
        match documents::delete(session, documents::abs_to_rel(&path), false) {
            Err(e) => warn!("Error while removing firestore document at {} - {}", &path, e),
            _ => {}
        };
    }

    Ok(String::new())
}

#[get("/update_stats", rank = 2)]
pub fn update_stats_unauthorized() -> MyResponder {
    MyResponder::access_denied("REQUIRES_AUTHORISATION","Requires authorization")
}

#[post("/addon", format = "application/json", data = "<request>")]
pub fn addon_put(
    request: Json<addons::AddonFileEntryPlusStats>,
    github_client: rocket::State<github::GithubClient>,
    oauth_user: guard_oauth_jwt_access::OAuthIdentity,
    _rate_limiter: RateLimiter,
) -> Result<(), MyResponder> {
    // Only the google account is allowed to call this endpoint
    if oauth_user.credentials_index != CREDENTIALS_OHX_SERVICE_ACCOUNT_INDEX || oauth_user.user_id.is_none() {
        return Err(MyResponder::access_denied("OHX_ACCOUNT_ONLY",
                                              "Only an OHX account is allowed to call this endpoint",
        ));
    }
    let user_id = oauth_user.user_id.unwrap();

    let (mut addons, sha) = github::get_data_content(&github_client)?;

    let addon = addons.get(&request.x_ohx_registry.id);
    let commit_reason = if let Some(addon) = addon {
        if addon.owner != user_id {
            return Err(MyResponder::bad_request("WRONG_OWNER",
                                                "You are not the author of this Addon",
            ));
        }

        let new_version = semver::Version::parse(&request.x_ohx_registry.version);
        let current_version = semver::Version::parse(&addon.entry.version)?;
        if let Ok(new_version) = new_version {
            if new_version <= current_version {
                return Err(MyResponder::bad_request("VERSION_MUST_BE_NEWER",
                                                    "You can only publish newer versions",
                ));
            }
        }
        format!("Updated Addon {}", &request.x_ohx_registry.id)
    } else {
        format!("Added Addon {}", &request.x_ohx_registry.id)
    };

    // Check if all "build" service entries have been replaced by an "image" service entry
    for (service_id, service) in &request.services {
        if service.build.is_some() || service.image.is_none() {
            return Err(MyResponder::bad_request("NOT_PREPROCESSED",
                                                &format!("The addon entry has not been preprocessed. The service '{}' requires an 'image'. 'build' is not allowed!", service_id),
            ));
        }
    }

    let (mut addon_detail, sha_details) = match github::get_data_detail_file(&github_client, &request.x_ohx_registry.id) {
        Ok(v) => v,
        Err(_) => (AddonDetailedInfo::default(), None)
    };

    addon_detail.services = request.services.clone();
    addon_detail.runtime = request.x_runtime.clone();
    addon_detail.archs = request.archs.clone();
    addon_detail.size = request.size;

    github::put_data_detail_file(&github_client, &request.x_ohx_registry.id, sha_details, &addon_detail, &commit_reason)?;

    addons.insert(request.x_ohx_registry.id.clone(), AddonRegistryEntry {
        entry: request.into_inner().x_ohx_registry,
        owner: user_id.clone(),
        last_updated: chrono::Utc::now().timestamp_millis(),
    });

    github::put_data_file(&github_client, &sha, &addons, &commit_reason)?;

    Ok(())
}

#[delete("/addon/<addon_id>?<force>")]
pub fn addon_delete(
    addon_id: String,
    force: Option<bool>,
    github_client: rocket::State<github::GithubClient>,
    oauth_user: guard_oauth_jwt_access::OAuthIdentity,
    _rate_limiter: RateLimiter,
) -> Result<(), MyResponder> {

    // Only the google account is allowed to call this endpoint
    if oauth_user.credentials_index != CREDENTIALS_OHX_SERVICE_ACCOUNT_INDEX || oauth_user.user_id.is_none() {
        return Err(MyResponder::access_denied("OHX_ACCOUNT_ONLY",
                                              "Only an OHX account is allowed to call this endpoint",
        ));
    }
    let user_id = oauth_user.user_id.unwrap();

    let (mut addons, sha) = github::get_data_content(&github_client)?;

    let addon = addons.get_mut(&addon_id);
    if let Some(addon) = addon {
        if addon.owner != user_id {
            return Err(MyResponder::bad_request("WRONG_OWNER",
                                                "You are not the author of this Addon",
            ));
        }

        addon.entry.status.code = addons::StatusCode::REMOVED;
        if let Some(force) = force {
            if force {
                addons.remove(&addon_id);
            }
        }

        github::put_data_file(&github_client, &sha, &addons, &format!("Addon removed: {}", &addon_id))?;
        return Ok(());
    }

    Err(MyResponder::bad_request("NOT_FOUND", "Addon not found"))
}

#[delete("/addon/<_addon_id>", rank = 2)]
pub fn addon_unauthorized(_addon_id: String) -> MyResponder {
    MyResponder::access_denied("REQUIRES_AUTHORISATION","Requires authorization")
}

#[post("/addon", rank = 2)]
pub fn addon_unauthorized2() -> MyResponder {
    MyResponder::access_denied("REQUIRES_AUTHORISATION","Requires authorization")
}
