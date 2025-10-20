use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Identifier for a user (email, sms, etc.)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Identifier {
    #[serde(rename = "type")]
    pub identifier_type: String,
    pub value: String,
}

impl Identifier {
    pub fn new(identifier_type: &str, value: &str) -> Self {
        Self {
            identifier_type: identifier_type.to_string(),
            value: value.to_string(),
        }
    }
}

/// Group information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Group {
    #[serde(rename = "type")]
    pub group_type: String,
    pub id: String,
    pub name: String,
}

impl Group {
    pub fn new(group_type: &str, id: &str, name: &str) -> Self {
        Self {
            group_type: group_type.to_string(),
            id: id.to_string(),
            name: name.to_string(),
        }
    }
}

/// Invitation target (email or sms)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InvitationTarget {
    #[serde(rename = "type")]
    pub target_type: String,
    pub value: String,
}

impl InvitationTarget {
    pub fn new(target_type: &str, value: &str) -> Self {
        Self {
            target_type: target_type.to_string(),
            value: value.to_string(),
        }
    }
}

/// Invitation acceptance information
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct InvitationAcceptance {
    pub id: String,
    pub account_id: String,
    pub project_id: String,
    pub accepted_at: String,
    pub target: InvitationTarget,
}

/// Full invitation details
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Invitation {
    pub id: String,
    pub account_id: String,
    pub click_throughs: u32,
    pub configuration_attributes: Option<HashMap<String, serde_json::Value>>,
    pub attributes: Option<HashMap<String, serde_json::Value>>,
    pub created_at: String,
    pub deactivated: bool,
    pub delivery_count: u32,
    pub delivery_types: Vec<String>,
    pub foreign_creator_id: String,
    pub invitation_type: String,
    pub modified_at: Option<String>,
    pub status: String,
    pub target: Vec<InvitationTarget>,
    pub views: u32,
    pub widget_configuration_id: String,
    pub project_id: String,
    pub groups: Vec<Group>,
    pub accepts: Vec<InvitationAcceptance>,
}

/// Response containing multiple invitations
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InvitationsResponse {
    pub invitations: Option<Vec<Invitation>>,
}
