use std::collections::{HashMap, HashSet};

#[derive(Debug)]
pub struct AccessScopes(pub HashMap<String, HashSet<String>>);

impl AccessScopes {
    pub fn new(json: &str) -> Result<AccessScopes, failure::Error> {
        Ok(AccessScopes(serde_json::from_str(json)?))
    }
}
