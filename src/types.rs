use serde::{Deserialize, Serialize};
use std::collections::HashMap;

// ============================================================================
// Enums for type-safe API values
// ============================================================================

/// Target type for invitation responses (who was invited)
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum InvitationTargetType {
    Email,
    Phone,
    Share,
    Internal,
}

/// Target type for creating invitations
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CreateInvitationTargetType {
    Email,
    Phone,
    Internal,
}

/// Type of invitation
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum InvitationType {
    SingleUse,
    MultiUse,
    Autojoin,
}

/// Current status of an invitation
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum InvitationStatus {
    Queued,
    Sending,
    Sent,
    Delivered,
    Accepted,
    Shared,
    Unfurled,
    AcceptedElsewhere,
}

/// Delivery type for invitations
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DeliveryType {
    Email,
    Phone,
    Share,
    Internal,
}

// ============================================================================
// Core types
// ============================================================================

/// User type for JWT generation
/// Optional fields: user_name (max 200 chars), user_avatar_url (HTTPS URL, max 2000 chars), admin_scopes
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct User {
    pub id: String,
    pub email: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub user_name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub user_avatar_url: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub admin_scopes: Option<Vec<String>>,
}

impl User {
    pub fn new(id: &str, email: &str) -> Self {
        Self {
            id: id.to_string(),
            email: email.to_string(),
            user_name: None,
            user_avatar_url: None,
            admin_scopes: None,
        }
    }

    pub fn with_user_name(mut self, name: &str) -> Self {
        self.user_name = Some(name.to_string());
        self
    }

    pub fn with_user_avatar_url(mut self, avatar_url: &str) -> Self {
        self.user_avatar_url = Some(avatar_url.to_string());
        self
    }

    pub fn with_admin_scopes(mut self, scopes: Vec<String>) -> Self {
        self.admin_scopes = Some(scopes);
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

/// Invitation target from API responses
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InvitationTarget {
    #[serde(rename = "type")]
    pub target_type: InvitationTargetType,
    pub value: String,
    /// Display name of the person being invited
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    /// Avatar URL for the person being invited (for display in invitation lists)
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(rename = "avatarUrl")]
    pub avatar_url: Option<String>,
}

impl InvitationTarget {
    pub fn new(target_type: InvitationTargetType, value: &str) -> Self {
        Self {
            target_type,
            value: value.to_string(),
            name: None,
            avatar_url: None,
        }
    }

    pub fn email(value: &str) -> Self {
        Self::new(InvitationTargetType::Email, value)
    }

    pub fn phone(value: &str) -> Self {
        Self::new(InvitationTargetType::Phone, value)
    }

    pub fn with_name(mut self, name: &str) -> Self {
        self.name = Some(name.to_string());
        self
    }

    pub fn with_avatar_url(mut self, avatar_url: &str) -> Self {
        self.avatar_url = Some(avatar_url.to_string());
        self
    }
}

/// User data for accepting invitations (preferred format)
///
/// At least one of email or phone must be provided.
///
/// # Example
///
/// ```
/// use vortex_sdk::AcceptUser;
///
/// // With email only
/// let user = AcceptUser::new().with_email("user@example.com");
///
/// // With email and name
/// let user = AcceptUser::new()
///     .with_email("user@example.com")
///     .with_name("John Doe");
///
/// // With all fields
/// let user = AcceptUser::new()
///     .with_email("user@example.com")
///     .with_phone("+1234567890")
///     .with_name("John Doe");
/// ```
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct AcceptUser {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub email: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub phone: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
}

impl AcceptUser {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_email(mut self, email: &str) -> Self {
        self.email = Some(email.to_string());
        self
    }

    pub fn with_phone(mut self, phone: &str) -> Self {
        self.phone = Some(phone.to_string());
        self
    }

    pub fn with_name(mut self, name: &str) -> Self {
        self.name = Some(name.to_string());
        self
    }
}

/// Invitation acceptance information
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct InvitationAcceptance {
    pub id: Option<String>,
    pub account_id: Option<String>,
    pub project_id: Option<String>,
    pub accepted_at: Option<String>,
    pub target: Option<InvitationTarget>,
}

/// Full invitation details
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Invitation {
    #[serde(default)]
    pub id: String,
    #[serde(default)]
    pub account_id: String,
    #[serde(default)]
    pub click_throughs: u32,
    pub configuration_attributes: Option<HashMap<String, serde_json::Value>>,
    pub attributes: Option<HashMap<String, serde_json::Value>>,
    #[serde(default)]
    pub created_at: String,
    #[serde(default)]
    pub deactivated: bool,
    #[serde(default)]
    pub delivery_count: u32,
    #[serde(default)]
    pub delivery_types: Vec<DeliveryType>,
    #[serde(default)]
    pub foreign_creator_id: String,
    pub invitation_type: InvitationType,
    pub modified_at: Option<String>,
    pub status: InvitationStatus,
    #[serde(default)]
    pub target: Vec<InvitationTarget>,
    #[serde(default)]
    pub views: u32,
    #[serde(default)]
    pub widget_configuration_id: String,
    #[serde(default)]
    pub project_id: String,
    #[serde(default)]
    pub groups: Vec<InvitationGroup>,
    #[serde(default)]
    pub accepts: Vec<InvitationAcceptance>,
    pub expired: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub expires: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub source: Option<String>,
    /// Customer-defined subtype for analytics segmentation (e.g., "pymk", "find-friends")
    #[serde(skip_serializing_if = "Option::is_none")]
    pub subtype: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub creator_name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub creator_avatar_url: Option<String>,
}

/// Response containing multiple invitations
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InvitationsResponse {
    pub invitations: Option<Vec<Invitation>>,
}

/// Accept invitation parameter - supports both new User format and legacy Target format
#[derive(Debug, Clone)]
pub enum AcceptInvitationParam {
    /// New User format (preferred)
    User(AcceptUser),
    /// Legacy target format (deprecated)
    Target(InvitationTarget),
    /// Legacy multiple targets format (deprecated)
    Targets(Vec<InvitationTarget>),
}

impl From<AcceptUser> for AcceptInvitationParam {
    fn from(user: AcceptUser) -> Self {
        AcceptInvitationParam::User(user)
    }
}

impl From<InvitationTarget> for AcceptInvitationParam {
    fn from(target: InvitationTarget) -> Self {
        AcceptInvitationParam::Target(target)
    }
}

impl From<Vec<InvitationTarget>> for AcceptInvitationParam {
    fn from(targets: Vec<InvitationTarget>) -> Self {
        AcceptInvitationParam::Targets(targets)
    }
}

// --- Types for creating invitations via backend API ---

/// Target for creating an invitation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateInvitationTarget {
    #[serde(rename = "type")]
    pub target_type: CreateInvitationTargetType,
    /// Target value: email address, phone number, or internal user ID
    pub value: String,
    /// Display name of the person being invited
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    /// Avatar URL for the person being invited (for display in invitation lists)
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(rename = "avatarUrl")]
    pub avatar_url: Option<String>,
}

impl CreateInvitationTarget {
    pub fn new(target_type: CreateInvitationTargetType, value: &str) -> Self {
        Self {
            target_type,
            value: value.to_string(),
            name: None,
            avatar_url: None,
        }
    }

    pub fn email(value: &str) -> Self {
        Self::new(CreateInvitationTargetType::Email, value)
    }

    pub fn phone(value: &str) -> Self {
        Self::new(CreateInvitationTargetType::Phone, value)
    }

    /// Alias for phone (backward compatibility)
    pub fn sms(value: &str) -> Self {
        Self::phone(value)
    }

    pub fn internal(value: &str) -> Self {
        Self::new(CreateInvitationTargetType::Internal, value)
    }

    pub fn with_name(mut self, name: &str) -> Self {
        self.name = Some(name.to_string());
        self
    }

    pub fn with_avatar_url(mut self, avatar_url: &str) -> Self {
        self.avatar_url = Some(avatar_url.to_string());
        self
    }
}

/// Information about the user creating the invitation (the inviter)
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Inviter {
    /// Required: Your internal user ID for the inviter
    pub user_id: String,
    /// Optional: Email of the inviter
    #[serde(skip_serializing_if = "Option::is_none")]
    pub user_email: Option<String>,
    /// Optional: Display name of the inviter
    #[serde(skip_serializing_if = "Option::is_none")]
    pub user_name: Option<String>,
    /// Optional: Avatar URL of the inviter
    #[serde(skip_serializing_if = "Option::is_none")]
    pub user_avatar_url: Option<String>,
}

impl Inviter {
    pub fn new(user_id: &str) -> Self {
        Self {
            user_id: user_id.to_string(),
            user_email: None,
            user_name: None,
            user_avatar_url: None,
        }
    }

    pub fn with_email(mut self, email: &str) -> Self {
        self.user_email = Some(email.to_string());
        self
    }

    pub fn with_user_name(mut self, name: &str) -> Self {
        self.user_name = Some(name.to_string());
        self
    }

    pub fn with_user_avatar_url(mut self, url: &str) -> Self {
        self.user_avatar_url = Some(url.to_string());
        self
    }
}

/// Group information for creating invitations
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateInvitationGroup {
    /// Group type (e.g., "team", "organization")
    #[serde(rename = "type")]
    pub group_type: String,
    /// Your internal group ID
    pub group_id: String,
    /// Display name of the group
    pub name: String,
}

impl CreateInvitationGroup {
    pub fn new(group_type: &str, group_id: &str, name: &str) -> Self {
        Self {
            group_type: group_type.to_string(),
            group_id: group_id.to_string(),
            name: name.to_string(),
        }
    }
}

/// Valid Open Graph types for unfurl configuration
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum UnfurlOgType {
    Website,
    Article,
    Video,
    Music,
    Book,
    Profile,
    Product,
}

/// Configuration for link unfurl (Open Graph) metadata
/// Controls how the invitation link appears when shared on social platforms or messaging apps
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct UnfurlConfig {
    /// The title shown in link previews (og:title)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub title: Option<String>,
    /// The description shown in link previews (og:description)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    /// The image URL shown in link previews (og:image) - must be HTTPS
    #[serde(skip_serializing_if = "Option::is_none")]
    pub image: Option<String>,
    /// The Open Graph type (og:type)
    #[serde(skip_serializing_if = "Option::is_none", rename = "type")]
    pub og_type: Option<UnfurlOgType>,
    /// The site name shown in link previews (og:site_name)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub site_name: Option<String>,
}

impl UnfurlConfig {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_title(mut self, title: &str) -> Self {
        self.title = Some(title.to_string());
        self
    }

    pub fn with_description(mut self, description: &str) -> Self {
        self.description = Some(description.to_string());
        self
    }

    pub fn with_image(mut self, image: &str) -> Self {
        self.image = Some(image.to_string());
        self
    }

    pub fn with_type(mut self, og_type: UnfurlOgType) -> Self {
        self.og_type = Some(og_type);
        self
    }

    pub fn with_site_name(mut self, site_name: &str) -> Self {
        self.site_name = Some(site_name.to_string());
        self
    }
}

/// Request body for creating an invitation
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateInvitationRequest {
    pub widget_configuration_id: String,
    pub target: CreateInvitationTarget,
    pub inviter: Inviter,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub groups: Option<Vec<CreateInvitationGroup>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub source: Option<String>,
    /// Customer-defined subtype for analytics segmentation (e.g., "pymk", "find-friends")
    #[serde(skip_serializing_if = "Option::is_none")]
    pub subtype: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub template_variables: Option<HashMap<String, String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub metadata: Option<HashMap<String, serde_json::Value>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub unfurl_config: Option<UnfurlConfig>,
}

impl CreateInvitationRequest {
    pub fn new(
        widget_configuration_id: &str,
        target: CreateInvitationTarget,
        inviter: Inviter,
    ) -> Self {
        Self {
            widget_configuration_id: widget_configuration_id.to_string(),
            target,
            inviter,
            groups: None,
            source: None,
            subtype: None,
            template_variables: None,
            metadata: None,
            unfurl_config: None,
        }
    }

    pub fn with_groups(mut self, groups: Vec<CreateInvitationGroup>) -> Self {
        self.groups = Some(groups);
        self
    }

    pub fn with_source(mut self, source: &str) -> Self {
        self.source = Some(source.to_string());
        self
    }

    pub fn with_subtype(mut self, subtype: &str) -> Self {
        self.subtype = Some(subtype.to_string());
        self
    }

    pub fn with_template_variables(mut self, vars: HashMap<String, String>) -> Self {
        self.template_variables = Some(vars);
        self
    }

    pub fn with_metadata(mut self, metadata: HashMap<String, serde_json::Value>) -> Self {
        self.metadata = Some(metadata);
        self
    }

    pub fn with_unfurl_config(mut self, unfurl_config: UnfurlConfig) -> Self {
        self.unfurl_config = Some(unfurl_config);
        self
    }
}

/// Response from creating an invitation
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateInvitationResponse {
    /// The ID of the created invitation
    pub id: String,
    /// The short link for the invitation
    pub short_link: String,
    /// The status of the invitation
    pub status: String,
    /// When the invitation was created
    pub created_at: String,
}
