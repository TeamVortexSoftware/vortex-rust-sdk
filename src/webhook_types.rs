use serde::{Deserialize, Serialize};
use std::collections::HashMap;

// ============================================================================
// Webhook Event Type Constants
// ============================================================================

/// Webhook event type constants for Vortex state changes.
pub mod webhook_event_type {
    // Invitation Lifecycle
    pub const INVITATION_CREATED: &str = "invitation.created";
    pub const INVITATION_ACCEPTED: &str = "invitation.accepted";
    pub const INVITATION_DEACTIVATED: &str = "invitation.deactivated";
    pub const INVITATION_EMAIL_DELIVERED: &str = "invitation.email.delivered";
    pub const INVITATION_EMAIL_BOUNCED: &str = "invitation.email.bounced";
    pub const INVITATION_EMAIL_OPENED: &str = "invitation.email.opened";
    pub const INVITATION_LINK_CLICKED: &str = "invitation.link.clicked";
    pub const INVITATION_REMINDER_SENT: &str = "invitation.reminder.sent";

    // Deployment Lifecycle
    pub const DEPLOYMENT_CREATED: &str = "deployment.created";
    pub const DEPLOYMENT_DEACTIVATED: &str = "deployment.deactivated";

    // A/B Testing
    pub const ABTEST_STARTED: &str = "abtest.started";
    pub const ABTEST_WINNER_DECLARED: &str = "abtest.winner_declared";

    // Member/Group
    pub const MEMBER_CREATED: &str = "member.created";
    pub const GROUP_MEMBER_ADDED: &str = "group.member.added";

    // Email
    pub const EMAIL_COMPLAINED: &str = "email.complained";
}

/// Analytics event type constants for behavioral telemetry.
pub mod analytics_event_type {
    pub const WIDGET_LOADED: &str = "widget_loaded";
    pub const INVITATION_SENT: &str = "invitation_sent";
    pub const INVITATION_CLICKED: &str = "invitation_clicked";
    pub const INVITATION_ACCEPTED: &str = "invitation_accepted";
    pub const SHARE_TRIGGERED: &str = "share_triggered";
}

// ============================================================================
// Event Types
// ============================================================================

/// A Vortex webhook event representing a server-side state change.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct VortexWebhookEvent {
    pub id: String,
    #[serde(rename = "type")]
    pub event_type: String,
    pub timestamp: String,
    pub account_id: String,
    pub environment_id: Option<String>,
    pub source_table: String,
    pub operation: String,
    pub data: HashMap<String, serde_json::Value>,
}

/// An analytics event representing client-side behavioral telemetry.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct VortexAnalyticsEvent {
    pub id: String,
    pub name: String,
    pub account_id: String,
    pub organization_id: String,
    pub project_id: String,
    pub environment_id: String,
    pub deployment_id: Option<String>,
    pub widget_configuration_id: Option<String>,
    pub foreign_user_id: Option<String>,
    pub session_id: Option<String>,
    pub payload: Option<HashMap<String, serde_json::Value>>,
    pub platform: Option<String>,
    pub segmentation: Option<String>,
    pub timestamp: String,
}

/// Any event delivered to a Vortex webhook endpoint.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum VortexEvent {
    Webhook(VortexWebhookEvent),
    Analytics(VortexAnalyticsEvent),
}

impl VortexEvent {
    /// Returns true if this is a webhook event.
    pub fn is_webhook_event(&self) -> bool {
        matches!(self, VortexEvent::Webhook(_))
    }

    /// Returns true if this is an analytics event.
    pub fn is_analytics_event(&self) -> bool {
        matches!(self, VortexEvent::Analytics(_))
    }

    /// Try to get the inner webhook event.
    pub fn as_webhook_event(&self) -> Option<&VortexWebhookEvent> {
        match self {
            VortexEvent::Webhook(e) => Some(e),
            _ => None,
        }
    }

    /// Try to get the inner analytics event.
    pub fn as_analytics_event(&self) -> Option<&VortexAnalyticsEvent> {
        match self {
            VortexEvent::Analytics(e) => Some(e),
            _ => None,
        }
    }
}
