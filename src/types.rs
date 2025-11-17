use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// User type for JWT generation
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct User {
    pub id: String,
    pub email: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub admin_scopes: Option<Vec<String>>,
    #[serde(flatten)]
    pub extra: Option<HashMap<String, serde_json::Value>>,
}

impl User {
    pub fn new(id: &str, email: &str) -> Self {
        Self {
            id: id.to_string(),
            email: email.to_string(),
            admin_scopes: None,
            extra: None,
        }
    }

    pub fn with_admin_scopes(mut self, scopes: Vec<String>) -> Self {
        self.admin_scopes = Some(scopes);
        self
    }

    pub fn with_extra(mut self, extra: HashMap<String, serde_json::Value>) -> Self {
        self.extra = Some(extra);
        self
    }
}

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

/// Group information for JWT generation (input)
/// Supports both 'id' (legacy) and 'groupId' (preferred) for backward compatibility
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Group {
    #[serde(rename = "type")]
    pub group_type: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub group_id: Option<String>,
    pub name: String,
}

impl Group {
    pub fn new(group_type: &str, name: &str) -> Self {
        Self {
            group_type: group_type.to_string(),
            id: None,
            group_id: None,
            name: name.to_string(),
        }
    }

    pub fn with_id(mut self, id: &str) -> Self {
        self.id = Some(id.to_string());
        self
    }

    pub fn with_group_id(mut self, group_id: &str) -> Self {
        self.group_id = Some(group_id.to_string());
        self
    }
}

/// Invitation group from API responses
/// This matches the MemberGroups table structure from the API
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct InvitationGroup {
    /// Vortex internal UUID
    pub id: String,
    /// Vortex account ID
    pub account_id: String,
    /// Customer's group ID (the ID they provided to Vortex)
    pub group_id: String,
    /// Group type (e.g., "workspace", "team")
    #[serde(rename = "type")]
    pub group_type: String,
    /// Group name
    pub name: String,
    /// ISO 8601 timestamp when the group was created
    pub created_at: String,
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
    pub groups: Vec<InvitationGroup>,
    pub accepts: Vec<InvitationAcceptance>,
}

/// Response containing multiple invitations
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InvitationsResponse {
    pub invitations: Option<Vec<Invitation>>,
}
