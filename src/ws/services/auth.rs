//! Authentication — login/register/user-management responses.

use serde::{Deserialize, Serialize};
use ts_rs::TS;

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export, export_to = "ws/auth.ts")]
pub struct AuthLoginResponseParams {
    pub ok: bool,
    #[serde(default)]
    #[ts(optional)]
    pub token: Option<String>,
    #[serde(default)]
    #[ts(optional)]
    pub session_id: Option<String>,
    #[serde(default)]
    #[ts(optional)]
    pub user_id: Option<String>,
    #[serde(default)]
    #[ts(optional)]
    pub username: Option<String>,
    #[serde(default)]
    #[ts(optional)]
    pub display_name: Option<String>,
    #[serde(default)]
    #[ts(optional)]
    pub role: Option<String>,
    #[serde(default)]
    #[ts(optional)]
    pub error: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export, export_to = "ws/auth.ts")]
pub struct AuthRegisterResponseParams {
    pub ok: bool,
    #[serde(default)]
    #[ts(optional)]
    pub user_id: Option<String>,
    #[serde(default)]
    #[ts(optional)]
    pub username: Option<String>,
    #[serde(default)]
    #[ts(optional)]
    pub error: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export, export_to = "ws/auth.ts")]
pub struct UserProfileSummary {
    pub id: String,
    pub username: String,
    pub email: String,
    pub display_name: String,
    pub role: String,
    pub is_active: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export, export_to = "ws/auth.ts")]
pub struct AuthListUsersResponseParams {
    pub ok: bool,
    #[serde(default)]
    #[ts(optional)]
    pub users: Option<Vec<UserProfileSummary>>,
    #[serde(default)]
    #[ts(optional)]
    pub error: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export, export_to = "ws/auth.ts")]
pub struct AuthGetUserResponseParams {
    pub ok: bool,
    #[serde(default)]
    #[ts(optional)]
    pub user: Option<UserProfileSummary>,
    #[serde(default)]
    #[ts(optional)]
    pub error: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export, export_to = "ws/auth.ts")]
pub struct AuthDeleteUserResponseParams {
    pub ok: bool,
    #[serde(default)]
    #[ts(optional)]
    pub error: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export, export_to = "ws/auth.ts")]
pub struct AuthChangePasswordResponseParams {
    pub ok: bool,
    #[serde(default)]
    #[ts(optional)]
    pub error: Option<String>,
}
