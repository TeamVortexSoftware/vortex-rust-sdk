use hmac::{Hmac, Mac};
use sha2::Sha256;

use crate::error::VortexError;
use crate::webhook_types::VortexEvent;

type HmacSha256 = Hmac<Sha256>;

/// Vortex webhook verification and parsing.
///
/// # Example
///
/// ```
/// use vortex_sdk::VortexWebhooks;
///
/// let webhooks = VortexWebhooks::new("whsec_your_secret").unwrap();
/// ```
pub struct VortexWebhooks {
    secret: String,
}

impl VortexWebhooks {
    /// Create a new webhook verifier with the given signing secret.
    ///
    /// # Errors
    ///
    /// Returns `VortexError::WebhookSignatureError` if the secret is empty.
    pub fn new(secret: impl Into<String>) -> Result<Self, VortexError> {
        let secret = secret.into();
        if secret.is_empty() {
            return Err(VortexError::WebhookSignatureError(
                "Webhook secret must not be empty.".into(),
            ));
        }
        Ok(Self { secret })
    }

    /// Verify the HMAC-SHA256 signature of an incoming webhook payload.
    ///
    /// Uses constant-time comparison to prevent timing attacks.
    pub fn verify_signature(&self, payload: &[u8], signature: &str) -> bool {
        let Ok(mut mac) = HmacSha256::new_from_slice(self.secret.as_bytes()) else {
            return false;
        };
        mac.update(payload);

        let expected = hex_encode(mac.finalize().into_bytes().as_slice());

        // Constant-time comparison
        constant_time_eq(expected.as_bytes(), signature.as_bytes())
    }

    /// Verify and parse an incoming webhook payload.
    ///
    /// Returns a typed `VortexEvent` on success, or a `VortexError::WebhookSignatureError`
    /// if the signature is invalid.
    ///
    /// # Arguments
    ///
    /// * `payload` - The raw request body bytes
    /// * `signature` - The value of the `X-Vortex-Signature` header
    pub fn construct_event(&self, payload: &[u8], signature: &str) -> Result<VortexEvent, VortexError> {
        if !self.verify_signature(payload, signature) {
            return Err(VortexError::WebhookSignatureError(
                "Webhook signature verification failed. Ensure you are using the raw request body and the correct signing secret.".into(),
            ));
        }

        serde_json::from_slice(payload).map_err(|e| {
            VortexError::SerializationError(format!("Failed to parse webhook payload: {}", e))
        })
    }
}

/// Hex-encode bytes (lowercase).
fn hex_encode(bytes: &[u8]) -> String {
    bytes.iter().map(|b| format!("{:02x}", b)).collect()
}

/// Constant-time byte comparison.
fn constant_time_eq(a: &[u8], b: &[u8]) -> bool {
    if a.len() != b.len() {
        return false;
    }
    let mut diff = 0u8;
    for (x, y) in a.iter().zip(b.iter()) {
        diff |= x ^ y;
    }
    diff == 0
}

#[cfg(test)]
mod tests {
    use super::*;

    const TEST_SECRET: &str = "whsec_test_secret";

    fn sign(payload: &[u8]) -> String {
        let mut mac = HmacSha256::new_from_slice(TEST_SECRET.as_bytes()).unwrap();
        mac.update(payload);
        hex_encode(mac.finalize().into_bytes().as_slice())
    }

    const SAMPLE_WEBHOOK: &str = r#"{"id":"evt_1","type":"invitation.accepted","timestamp":"2026-02-25T12:00:00Z","accountId":"acc_1","environmentId":null,"sourceTable":"invitations","operation":"update","data":{"targetEmail":"user@test.com"}}"#;

    const SAMPLE_ANALYTICS: &str = r#"{"id":"ae_1","name":"widget_loaded","accountId":"acc_1","organizationId":"org_1","projectId":"proj_1","environmentId":"env_1","deploymentId":null,"widgetConfigurationId":null,"foreignUserId":null,"sessionId":null,"payload":null,"platform":"web","segmentation":null,"timestamp":"2026-02-25T12:00:00Z"}"#;

    #[test]
    fn test_verify_valid_signature() {
        let webhooks = VortexWebhooks::new(TEST_SECRET).unwrap();
        let sig = sign(SAMPLE_WEBHOOK.as_bytes());
        assert!(webhooks.verify_signature(SAMPLE_WEBHOOK.as_bytes(), &sig));
    }

    #[test]
    fn test_verify_invalid_signature() {
        let webhooks = VortexWebhooks::new(TEST_SECRET).unwrap();
        assert!(!webhooks.verify_signature(SAMPLE_WEBHOOK.as_bytes(), "bad_sig"));
    }

    #[test]
    fn test_verify_tampered_payload() {
        let webhooks = VortexWebhooks::new(TEST_SECRET).unwrap();
        let sig = sign(SAMPLE_WEBHOOK.as_bytes());
        let tampered = SAMPLE_WEBHOOK.replace("evt_1", "evt_hacked");
        assert!(!webhooks.verify_signature(tampered.as_bytes(), &sig));
    }

    #[test]
    fn test_construct_webhook_event() {
        let webhooks = VortexWebhooks::new(TEST_SECRET).unwrap();
        let sig = sign(SAMPLE_WEBHOOK.as_bytes());
        let event = webhooks.construct_event(SAMPLE_WEBHOOK.as_bytes(), &sig).unwrap();
        assert!(event.is_webhook_event());
        let wh = event.as_webhook_event().unwrap();
        assert_eq!(wh.event_type, "invitation.accepted");
    }

    #[test]
    fn test_construct_analytics_event() {
        let webhooks = VortexWebhooks::new(TEST_SECRET).unwrap();
        let sig = sign(SAMPLE_ANALYTICS.as_bytes());
        let event = webhooks.construct_event(SAMPLE_ANALYTICS.as_bytes(), &sig).unwrap();
        assert!(event.is_analytics_event());
        let ae = event.as_analytics_event().unwrap();
        assert_eq!(ae.name, "widget_loaded");
    }

    #[test]
    fn test_construct_event_invalid_signature() {
        let webhooks = VortexWebhooks::new(TEST_SECRET).unwrap();
        let result = webhooks.construct_event(SAMPLE_WEBHOOK.as_bytes(), "bad");
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), VortexError::WebhookSignatureError(_)));
    }
}
