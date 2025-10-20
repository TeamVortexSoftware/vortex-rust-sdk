use base64::{engine::general_purpose::URL_SAFE_NO_PAD, Engine};
use hmac::{Hmac, Mac};
use reqwest::{Client as HttpClient, RequestBuilder, Response};
use serde_json::json;
use sha2::Sha256;
use std::collections::HashMap;
use std::time::{SystemTime, UNIX_EPOCH};
use uuid::Uuid;

use crate::error::VortexError;
use crate::types::*;

type HmacSha256 = Hmac<Sha256>;

/// Vortex Rust SDK Client
///
/// Provides JWT generation and Vortex API integration for Rust applications.
/// Compatible with React providers and follows the same paradigms as other Vortex SDKs.
pub struct VortexClient {
    api_key: String,
    base_url: String,
    http_client: HttpClient,
}

impl VortexClient {
    /// Create a new Vortex client
    ///
    /// # Arguments
    ///
    /// * `api_key` - Your Vortex API key
    ///
    /// # Example
    ///
    /// ```
    /// use vortex_sdk::VortexClient;
    ///
    /// let client = VortexClient::new(std::env::var("VORTEX_API_KEY").unwrap());
    /// ```
    pub fn new(api_key: String) -> Self {
        let base_url = std::env::var("VORTEX_API_BASE_URL")
            .unwrap_or_else(|_| "https://api.vortexsoftware.com".to_string());

        Self {
            api_key,
            base_url,
            http_client: HttpClient::new(),
        }
    }

    /// Create a new Vortex client with a custom base URL
    ///
    /// # Arguments
    ///
    /// * `api_key` - Your Vortex API key
    /// * `base_url` - Custom base URL for the Vortex API
    pub fn with_base_url(api_key: String, base_url: String) -> Self {
        Self {
            api_key,
            base_url,
            http_client: HttpClient::new(),
        }
    }

    /// Generate a JWT token for the given user data
    ///
    /// This uses the same algorithm as the Node.js SDK to ensure
    /// complete compatibility with React providers.
    ///
    /// # Arguments
    ///
    /// * `user_id` - Unique identifier for the user
    /// * `identifiers` - List of identifiers (email, sms)
    /// * `groups` - List of groups the user belongs to
    /// * `role` - Optional user role
    ///
    /// # Example
    ///
    /// ```
    /// use vortex_sdk::{VortexClient, Identifier, Group};
    ///
    /// let client = VortexClient::new("your-api-key".to_string());
    /// let jwt = client.generate_jwt(
    ///     "user-123",
    ///     vec![Identifier::new("email", "user@example.com")],
    ///     vec![Group::new("team", "team-1", "Engineering")],
    ///     Some("admin")
    /// ).unwrap();
    /// ```
    pub fn generate_jwt(
        &self,
        user_id: &str,
        identifiers: Vec<Identifier>,
        groups: Vec<Group>,
        role: Option<&str>,
    ) -> Result<String, VortexError> {
        // Parse API key: format is VRTX.base64encodedId.key
        let parts: Vec<&str> = self.api_key.split('.').collect();
        if parts.len() != 3 {
            return Err(VortexError::InvalidApiKey(
                "Invalid API key format".to_string(),
            ));
        }

        let prefix = parts[0];
        let encoded_id = parts[1];
        let key = parts[2];

        if prefix != "VRTX" {
            return Err(VortexError::InvalidApiKey(
                "Invalid API key prefix".to_string(),
            ));
        }

        // Decode the UUID from base64url
        let id_bytes = URL_SAFE_NO_PAD
            .decode(encoded_id)
            .map_err(|e| VortexError::InvalidApiKey(format!("Failed to decode ID: {}", e)))?;

        if id_bytes.len() != 16 {
            return Err(VortexError::InvalidApiKey("ID must be 16 bytes".to_string()));
        }

        let uuid = Uuid::from_slice(&id_bytes)
            .map_err(|e| VortexError::InvalidApiKey(format!("Invalid UUID: {}", e)))?;
        let uuid_str = uuid.to_string();

        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();
        let expires = now + 3600; // 1 hour from now

        // Step 1: Derive signing key from API key + ID
        let mut hmac = HmacSha256::new_from_slice(key.as_bytes())
            .map_err(|e| VortexError::CryptoError(format!("HMAC error: {}", e)))?;
        hmac.update(uuid_str.as_bytes());
        let signing_key = hmac.finalize().into_bytes();

        // Step 2: Build header + payload (same structure as Node.js)
        let header = json!({
            "iat": now,
            "alg": "HS256",
            "typ": "JWT",
            "kid": uuid_str,
        });

        let payload = json!({
            "userId": user_id,
            "identifiers": identifiers,
            "groups": groups,
            "role": role,
            "expires": expires,
        });

        // Step 3: Base64URL encode header and payload
        let header_b64 = URL_SAFE_NO_PAD.encode(serde_json::to_vec(&header).unwrap());
        let payload_b64 = URL_SAFE_NO_PAD.encode(serde_json::to_vec(&payload).unwrap());

        // Step 4: Sign with HMAC-SHA256
        let to_sign = format!("{}.{}", header_b64, payload_b64);
        let mut sig_hmac = HmacSha256::new_from_slice(&signing_key)
            .map_err(|e| VortexError::CryptoError(format!("HMAC error: {}", e)))?;
        sig_hmac.update(to_sign.as_bytes());
        let signature = sig_hmac.finalize().into_bytes();
        let sig_b64 = URL_SAFE_NO_PAD.encode(&signature);

        Ok(format!("{}.{}.{}", header_b64, payload_b64, sig_b64))
    }

    /// Get invitations by target (email or sms)
    pub async fn get_invitations_by_target(
        &self,
        target_type: &str,
        target_value: &str,
    ) -> Result<Vec<Invitation>, VortexError> {
        let mut params = HashMap::new();
        params.insert("targetType", target_type);
        params.insert("targetValue", target_value);

        let response: InvitationsResponse = self
            .api_request("GET", "/api/v1/invitations", None::<&()>, Some(params))
            .await?;

        Ok(response.invitations.unwrap_or_default())
    }

    /// Get a specific invitation by ID
    pub async fn get_invitation(&self, invitation_id: &str) -> Result<Invitation, VortexError> {
        self.api_request(
            "GET",
            &format!("/api/v1/invitations/{}", invitation_id),
            None::<&()>,
            None,
        )
        .await
    }

    /// Revoke (delete) an invitation
    pub async fn revoke_invitation(&self, invitation_id: &str) -> Result<(), VortexError> {
        self.api_request::<(), ()>(
            "DELETE",
            &format!("/api/v1/invitations/{}", invitation_id),
            None,
            None,
        )
        .await?;
        Ok(())
    }

    /// Accept multiple invitations
    pub async fn accept_invitations(
        &self,
        invitation_ids: Vec<String>,
        target: InvitationTarget,
    ) -> Result<Invitation, VortexError> {
        let body = json!({
            "invitationIds": invitation_ids,
            "target": target,
        });

        self.api_request("POST", "/api/v1/invitations/accept", Some(&body), None)
            .await
    }

    /// Delete all invitations for a specific group
    pub async fn delete_invitations_by_group(
        &self,
        group_type: &str,
        group_id: &str,
    ) -> Result<(), VortexError> {
        self.api_request::<(), ()>(
            "DELETE",
            &format!("/api/v1/invitations/by-group/{}/{}", group_type, group_id),
            None,
            None,
        )
        .await?;
        Ok(())
    }

    /// Get all invitations for a specific group
    pub async fn get_invitations_by_group(
        &self,
        group_type: &str,
        group_id: &str,
    ) -> Result<Vec<Invitation>, VortexError> {
        let response: InvitationsResponse = self
            .api_request(
                "GET",
                &format!("/api/v1/invitations/by-group/{}/{}", group_type, group_id),
                None::<&()>,
                None,
            )
            .await?;

        Ok(response.invitations.unwrap_or_default())
    }

    /// Reinvite a user (send invitation again)
    pub async fn reinvite(&self, invitation_id: &str) -> Result<Invitation, VortexError> {
        self.api_request(
            "POST",
            &format!("/api/v1/invitations/{}/reinvite", invitation_id),
            None::<&()>,
            None,
        )
        .await
    }

    async fn api_request<T, B>(
        &self,
        method: &str,
        path: &str,
        body: Option<&B>,
        query_params: Option<HashMap<&str, &str>>,
    ) -> Result<T, VortexError>
    where
        T: serde::de::DeserializeOwned,
        B: serde::Serialize,
    {
        let url = format!("{}{}", self.base_url, path);

        let mut request = match method {
            "GET" => self.http_client.get(&url),
            "POST" => self.http_client.post(&url),
            "PUT" => self.http_client.put(&url),
            "DELETE" => self.http_client.delete(&url),
            _ => return Err(VortexError::InvalidRequest("Invalid HTTP method".to_string())),
        };

        // Add headers
        request = request
            .header("Content-Type", "application/json")
            .header("x-api-key", &self.api_key)
            .header("User-Agent", "vortex-rust-sdk/1.0.0");

        // Add query parameters
        if let Some(params) = query_params {
            request = request.query(&params);
        }

        // Add body
        if let Some(b) = body {
            request = request.json(b);
        }

        let response = request
            .send()
            .await
            .map_err(|e| VortexError::HttpError(e.to_string()))?;

        if !response.status().is_success() {
            let status = response.status();
            let error_text = response
                .text()
                .await
                .unwrap_or_else(|_| "Unknown error".to_string());
            return Err(VortexError::ApiError(format!(
                "API request failed: {} - {}",
                status, error_text
            )));
        }

        let text = response
            .text()
            .await
            .map_err(|e| VortexError::HttpError(e.to_string()))?;

        // Handle empty responses
        if text.is_empty() {
            return serde_json::from_str("{}")
                .map_err(|e| VortexError::SerializationError(e.to_string()));
        }

        serde_json::from_str(&text)
            .map_err(|e| VortexError::SerializationError(e.to_string()))
    }
}
