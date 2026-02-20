use base64::{engine::general_purpose::URL_SAFE_NO_PAD, Engine};
use hmac::{Hmac, Mac};
use reqwest::Client as HttpClient;
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
    /// let api_key = "VRTX.your_encoded_id.your_key".to_string();
    /// let client = VortexClient::new(api_key);
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

    /// Generate a JWT token for a user
    ///
    /// # Arguments
    ///
    /// * `user` - User object with id, email, and optional fields:
    ///   - name: user's display name (max 200 characters)
    ///   - avatar_url: user's avatar URL (must be HTTPS, max 2000 characters)
    ///   - admin_scopes: list of admin scopes (e.g., vec!["autojoin"])
    /// * `extra` - Optional additional properties to include in the JWT payload
    ///
    /// # Example
    ///
    /// ```
    /// use vortex_sdk::{VortexClient, User};
    /// use std::collections::HashMap;
    ///
    /// let client = VortexClient::new("VRTX.AAAAAAAAAAAAAAAAAAAAAA.test_secret_key".to_string());
    ///
    /// // Simple usage
    /// let user = User::new("user-123", "user@example.com")
    ///     .with_user_name("Jane Doe")                                     // Optional: user's display name
    ///     .with_user_avatar_url("https://example.com/avatars/jane.jpg")  // Optional: user's avatar URL
    ///     .with_admin_scopes(vec!["autojoin".to_string()]);         // Optional: grants admin privileges
    /// let jwt = client.generate_jwt(&user, None).unwrap();
    ///
    /// // With additional properties
    /// let mut extra = HashMap::new();
    /// extra.insert("role".to_string(), serde_json::json!("admin"));
    /// let jwt = client.generate_jwt(&user, Some(extra)).unwrap();
    /// ```
    pub fn generate_jwt(
        &self,
        user: &User,
        extra: Option<HashMap<String, serde_json::Value>>,
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

        // Step 2: Build header + payload
        let header = json!({
            "iat": now,
            "alg": "HS256",
            "typ": "JWT",
            "kid": uuid_str,
        });

        // Build payload with user data
        let mut payload_json = json!({
            "userId": user.id,
            "userEmail": user.email,
            "expires": expires,
        });

        // Add name if present
        if let Some(ref user_name) = user.user_name {
            payload_json["userName"] = json!(user_name);
        }

        // Add userAvatarUrl if present
        if let Some(ref user_avatar_url) = user.user_avatar_url {
            payload_json["userAvatarUrl"] = json!(user_avatar_url);
        }

        // Add adminScopes if present
        if let Some(ref scopes) = user.admin_scopes {
            payload_json["adminScopes"] = json!(scopes);
        }

        // Add allowedEmailDomains if present (for domain-restricted invitations)
        if let Some(ref domains) = user.allowed_email_domains {
            if !domains.is_empty() {
                payload_json["allowedEmailDomains"] = json!(domains);
            }
        }

        // Add any additional properties from extra parameter
        if let Some(extra_props) = extra {
            for (key, value) in extra_props {
                payload_json[key] = value;
            }
        }

        // Step 3: Base64URL encode header and payload
        let header_b64 = URL_SAFE_NO_PAD.encode(serde_json::to_vec(&header).unwrap());
        let payload_b64 = URL_SAFE_NO_PAD.encode(serde_json::to_vec(&payload_json).unwrap());

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
    ///
    /// # Arguments
    ///
    /// * `invitation_ids` - Vector of invitation IDs to accept
    /// * `param` - User data (preferred) or legacy target format
    ///
    /// # New User Format (Preferred)
    ///
    /// ```
    /// use vortex_sdk::{VortexClient, AcceptUser};
    ///
    /// # async fn example() {
    /// let client = VortexClient::new("VRTX.key.secret".to_string());
    /// let user = AcceptUser::new().with_email("user@example.com");
    /// let result = client.accept_invitations(vec!["inv-123".to_string()], user).await;
    /// # }
    /// ```
    ///
    /// # Legacy Target Format (Deprecated)
    ///
    /// ```
    /// use vortex_sdk::{VortexClient, InvitationTarget};
    ///
    /// # async fn example() {
    /// let client = VortexClient::new("VRTX.key.secret".to_string());
    /// let target = InvitationTarget::email("user@example.com");
    /// let result = client.accept_invitations(vec!["inv-123".to_string()], target).await;
    /// # }
    /// ```
    pub async fn accept_invitations(
        &self,
        invitation_ids: Vec<String>,
        param: impl Into<crate::types::AcceptInvitationParam>,
    ) -> Result<Invitation, VortexError> {
        use crate::types::{AcceptInvitationParam, AcceptUser};

        let param = param.into();

        // Convert all parameter types to User format to avoid async recursion
        let user = match param {
            AcceptInvitationParam::Targets(targets) => {
                eprintln!("[Vortex SDK] DEPRECATED: Passing a vector of targets is deprecated. Use the AcceptUser format and call once per user instead.");

                if targets.is_empty() {
                    return Err(VortexError::InvalidRequest("No targets provided".to_string()));
                }

                let mut last_result = None;
                let mut last_error = None;

                for target in targets {
                    // Convert target to user
                    let user = match target.target_type {
                        InvitationTargetType::Email => AcceptUser::new().with_email(&target.value),
                        InvitationTargetType::Phone => AcceptUser::new().with_phone(&target.value),
                        _ => AcceptUser::new().with_email(&target.value),
                    };

                    match Box::pin(self.accept_invitations(invitation_ids.clone(), user)).await {
                        Ok(result) => last_result = Some(result),
                        Err(e) => last_error = Some(e),
                    }
                }

                if let Some(err) = last_error {
                    return Err(err);
                }

                return last_result.ok_or_else(|| VortexError::InvalidRequest("No results".to_string()));
            }
            AcceptInvitationParam::Target(target) => {
                eprintln!("[Vortex SDK] DEPRECATED: Passing an InvitationTarget is deprecated. Use the AcceptUser format instead: AcceptUser::new().with_email(\"user@example.com\")");

                // Convert target to User format
                match target.target_type {
                    InvitationTargetType::Email => AcceptUser::new().with_email(&target.value),
                    InvitationTargetType::Phone => AcceptUser::new().with_phone(&target.value),
                    _ => AcceptUser::new().with_email(&target.value), // Default to email
                }
            }
            AcceptInvitationParam::User(user) => user,
        };

        // Validate that either email or phone is provided
        if user.email.is_none() && user.phone.is_none() {
            return Err(VortexError::InvalidRequest(
                "User must have either email or phone".to_string(),
            ));
        }

        let body = json!({
            "invitationIds": invitation_ids,
            "user": user,
        });

        self.api_request("POST", "/api/v1/invitations/accept", Some(&body), None)
            .await
    }

    /// Accept a single invitation (recommended method)
    ///
    /// This is the recommended method for accepting invitations.
    ///
    /// # Arguments
    ///
    /// * `invitation_id` - Single invitation ID to accept
    /// * `user` - User object with email and/or phone
    ///
    /// # Returns
    ///
    /// * `Result<Invitation, VortexError>` - The accepted invitation result
    ///
    /// # Example
    ///
    /// ```
    /// use vortex_sdk::{VortexClient, AcceptUser};
    ///
    /// # async fn example() {
    /// let client = VortexClient::new("VRTX.key.secret".to_string());
    /// let user = AcceptUser::new().with_email("user@example.com");
    /// let result = client.accept_invitation("inv-123", user).await;
    /// # }
    /// ```
    pub async fn accept_invitation(
        &self,
        invitation_id: &str,
        user: crate::types::AcceptUser,
    ) -> Result<Invitation, VortexError> {
        self.accept_invitations(vec![invitation_id.to_string()], user).await
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

    /// Create an invitation from your backend
    ///
    /// This method allows you to create invitations programmatically using your API key,
    /// without requiring a user JWT token. Useful for server-side invitation creation,
    /// such as "People You May Know" flows or admin-initiated invitations.
    ///
    /// # Target types
    ///
    /// - `email`: Send an email invitation
    /// - `sms`: Create an SMS invitation (short link returned for you to send)
    /// - `internal`: Create an internal invitation for PYMK flows (no email sent)
    ///
    /// # Example
    ///
    /// ```no_run
    /// use vortex_sdk::{VortexClient, CreateInvitationRequest, CreateInvitationTarget, Inviter, CreateInvitationGroup};
    ///
    /// #[tokio::main]
    /// async fn main() -> Result<(), Box<dyn std::error::Error>> {
    ///     let client = VortexClient::new("VRTX.xxx.yyy".to_string());
    ///
    ///     // Create an email invitation
    ///     let request = CreateInvitationRequest::new(
    ///         "widget-config-123",
    ///         CreateInvitationTarget::email("invitee@example.com"),
    ///         Inviter::new("user-456")
    ///             .with_email("inviter@example.com")
    ///             .with_user_name("John Doe"),
    ///     )
    ///     .with_groups(vec![
    ///         CreateInvitationGroup::new("team", "team-789", "Engineering"),
    ///     ]);
    ///
    ///     let result = client.create_invitation(&request).await?;
    ///
    ///     // Create an internal invitation (PYMK flow - no email sent)
    ///     let request = CreateInvitationRequest::new(
    ///         "widget-config-123",
    ///         CreateInvitationTarget::internal("internal-user-abc"),
    ///         Inviter::new("user-456"),
    ///     )
    ///     .with_source("pymk");
    ///
    ///     let result = client.create_invitation(&request).await?;
    ///     Ok(())
    /// }
    /// ```
    pub async fn create_invitation(
        &self,
        request: &CreateInvitationRequest,
    ) -> Result<CreateInvitationResponse, VortexError> {
        self.api_request("POST", "/api/v1/invitations", Some(request), None)
            .await
    }

    /// Get autojoin domains configured for a specific scope
    ///
    /// # Arguments
    ///
    /// * `scope_type` - The type of scope (e.g., "organization", "team", "project")
    /// * `scope` - The scope identifier (customer's group ID)
    ///
    /// # Returns
    ///
    /// AutojoinDomainsResponse with autojoin domains and associated invitation
    ///
    /// # Example
    ///
    /// ```no_run
    /// use vortex_sdk::VortexClient;
    ///
    /// #[tokio::main]
    /// async fn main() -> Result<(), Box<dyn std::error::Error>> {
    ///     let client = VortexClient::new("VRTX.your_key_here".to_string());
    ///
    ///     let result = client.get_autojoin_domains("organization", "acme-org").await?;
    ///     for domain in &result.autojoin_domains {
    ///         println!("Domain: {}", domain.domain);
    ///     }
    ///     Ok(())
    /// }
    /// ```
    pub async fn get_autojoin_domains(
        &self,
        scope_type: &str,
        scope: &str,
    ) -> Result<AutojoinDomainsResponse, VortexError> {
        let encoded_scope_type = urlencoding::encode(scope_type);
        let encoded_scope = urlencoding::encode(scope);
        let path = format!(
            "/api/v1/invitations/by-scope/{}/{}/autojoin",
            encoded_scope_type, encoded_scope
        );
        self.api_request::<AutojoinDomainsResponse, ()>("GET", &path, None, None)
            .await
    }

    /// Configure autojoin domains for a specific scope
    ///
    /// This endpoint syncs autojoin domains - it will add new domains, remove domains
    /// not in the provided list, and deactivate the autojoin invitation if all domains
    /// are removed (empty array).
    ///
    /// # Arguments
    ///
    /// * `request` - The configure autojoin request
    ///
    /// # Returns
    ///
    /// AutojoinDomainsResponse with updated autojoin domains and associated invitation
    ///
    /// # Example
    ///
    /// ```no_run
    /// use vortex_sdk::{VortexClient, ConfigureAutojoinRequest};
    ///
    /// #[tokio::main]
    /// async fn main() -> Result<(), Box<dyn std::error::Error>> {
    ///     let client = VortexClient::new("VRTX.your_key_here".to_string());
    ///
    ///     let request = ConfigureAutojoinRequest::new(
    ///         "acme-org",
    ///         "organization",
    ///         vec!["acme.com".to_string(), "acme.org".to_string()],
    ///         "widget-123",
    ///     )
    ///     .with_scope_name("Acme Corporation");
    ///
    ///     let result = client.configure_autojoin(&request).await?;
    ///     Ok(())
    /// }
    /// ```
    ///
    /// Sync an internal invitation action (accept or decline)
    ///
    /// This method notifies Vortex that an internal invitation was accepted or declined
    /// within your application, so Vortex can update the invitation status accordingly.
    ///
    /// # Example
    ///
    /// ```no_run
    /// use vortex_sdk::{VortexClient, SyncInternalInvitationRequest};
    ///
    /// #[tokio::main]
    /// async fn main() -> Result<(), Box<dyn std::error::Error>> {
    ///     let client = VortexClient::new("VRTX.xxx.yyy".to_string());
    ///     let request = SyncInternalInvitationRequest::new(
    ///         "user-123", "user-456", "accepted", "component-uuid",
    ///     );
    ///     let result = client.sync_internal_invitation(&request).await?;
    ///     println!("Processed {} invitations", result.processed);
    ///     Ok(())
    /// }
    /// ```
    pub async fn sync_internal_invitation(
        &self,
        request: &SyncInternalInvitationRequest,
    ) -> Result<SyncInternalInvitationResponse, VortexError> {
        self.api_request(
            "POST",
            "/api/v1/invitation-actions/sync-internal-invitation",
            Some(request),
            None,
        )
        .await
    }

    pub async fn configure_autojoin(
        &self,
        request: &ConfigureAutojoinRequest,
    ) -> Result<AutojoinDomainsResponse, VortexError> {
        self.api_request("POST", "/api/v1/invitations/autojoin", Some(request), None)
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
            .header("User-Agent", format!("vortex-rust-sdk/{}", env!("CARGO_PKG_VERSION")))
            .header("x-vortex-sdk-name", "vortex-rust-sdk")
            .header("x-vortex-sdk-version", env!("CARGO_PKG_VERSION"));

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
