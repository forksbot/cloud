use serde::{Deserialize, Serialize};

/// Users id, email, display name and a few more information
#[allow(non_snake_case)]
#[derive(Debug, Default, Deserialize, Serialize)]
pub struct OHXAuthUser {
    pub localId: Option<String>,
    pub email: Option<String>,
    pub displayName: Option<String>,
}

