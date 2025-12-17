# Vortex Rust SDK Integration Guide

## SDK Information

**Crate**: `vortex-sdk`
**Type**: Base SDK (Core library for Rust applications)
**Framework**: Framework-agnostic Rust (edition 2021)
**Integration Style**: Client-based with async/await support

This SDK provides the core `VortexClient` struct for Rust applications. It works with:
- **Axum** - Modern async web framework with ergonomic extractors
- **Actix Web** - High-performance actor-based framework
- **Rocket** - Type-safe web framework
- **Warp** - Lightweight async web server
- **Plain Rust** - Standalone applications or custom frameworks
- **Any Rust framework** - Framework-agnostic design

**Key Features**:
- **Type-safe**: Full type safety with Rust's type system
- **Async/await**: Built on tokio for efficient async operations
- **Zero-copy where possible**: Efficient memory usage
- **reqwest-based HTTP**: Modern async HTTP client
- **Comprehensive error handling**: Rich error types with `VortexError`

---

## Expected Input Context

When this guide is invoked by the orchestrator, expect:

### Integration Contract
```typescript
{
  backend: {
    framework: 'rust',
    packageManager: 'cargo',
    rustVersion: string,  // e.g., '1.70', '1.75', '1.80'
    frameworkDetails?: {
      name?: 'axum' | 'actix-web' | 'rocket' | 'warp' | 'custom',
      version?: string,
      isAsync?: boolean
    }
  }
}
```

### Discovery Data
```typescript
{
  projectRoot: string,
  existingFiles: string[],  // Cargo.toml, src/main.rs, etc.
  hasCargoToml: boolean,
  frameworkName?: string
}
```

---

## Implementation Overview

The Rust SDK provides the core `VortexClient` struct for:
1. **JWT Generation**: Generate JWTs for authenticated users with custom attributes
2. **Invitation Management**: Query, accept, revoke, and manage invitations
3. **Async Operations**: All API methods are async with tokio
4. **Type Safety**: Full type safety with Rust's type system

Integration involves:
1. Adding the crate to `Cargo.toml`
2. Creating a `VortexClient` instance with your API key
3. Implementing HTTP handlers that call Vortex methods
4. Extracting authenticated user from your auth system
5. **Critical**: Implementing custom database logic for accepting invitations

---

## Critical SDK Specifics

### 1. Client Instantiation
```rust
use vortex_sdk::VortexClient;

let client = VortexClient::new(
    std::env::var("VORTEX_API_KEY").unwrap()
);

// Or with custom base URL
let client = VortexClient::with_base_url(
    std::env::var("VORTEX_API_KEY").unwrap(),
    "https://api.vortexsoftware.com".to_string()
);
```

### 2. JWT Generation - Current Format (Recommended)
```rust
use vortex_sdk::User;

let user = User::new("user-123", "user@example.com")
    .with_admin_scopes(vec!["autojoin".to_string()]);

let jwt = client.generate_jwt(&user, None)?;
```

### 3. JWT Generation with Additional Properties
```rust
use std::collections::HashMap;
use serde_json::json;

let user = User::new("user-123", "user@example.com");

let mut extra = HashMap::new();
extra.insert("componentId".to_string(), json!("optional-component-id"));
extra.insert("scope".to_string(), json!("optional-scope"));
extra.insert("scopeType".to_string(), json!("optional-scope-type"));

let jwt = client.generate_jwt(&user, Some(extra))?;
```

### 4. Async API Methods
```rust
// All API methods are async and return Result<T, VortexError>

// Get invitations by target
let invitations = client
    .get_invitations_by_target("email", "user@example.com")
    .await?;

// Get single invitation
let invitation = client.get_invitation("invitation-id").await?;

// Get invitations by group
let invitations = client
    .get_invitations_by_group("team", "team-123")
    .await?;
```

### 5. Invitation Operations
```rust
// Revoke invitation
client.revoke_invitation("invitation-id").await?;

// Resend invitation
let invitation = client.reinvite("invitation-id").await?;

// Delete all invitations for a group
client.delete_invitations_by_group("team", "team-123").await?;
```

### 6. Accept Invitations - REQUIRES DATABASE OVERRIDE
```rust
use vortex_sdk::InvitationTarget;

// This SDK method just marks invitations as accepted in Vortex
// YOU MUST implement database logic to actually add user to groups
let target = InvitationTarget::new("email", "user@example.com");
client.accept_invitations(
    vec!["inv-1".to_string(), "inv-2".to_string()],
    target
).await?;

// After calling this, you MUST implement your own database logic:
// - Add user to teams/organizations
// - Grant permissions/roles
// - Create user records if needed
// - Update user metadata
```

---

## Step-by-Step Implementation

### Step 1: Add Dependency

Add to your `Cargo.toml`:
```toml
[dependencies]
vortex-sdk = "1.0"
tokio = { version = "1.0", features = ["full"] }
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"

# Choose your web framework
axum = "0.7"  # For Axum
# OR
actix-web = "4.0"  # For Actix Web
# OR
rocket = "0.5"  # For Rocket
```

Then run:
```bash
cargo build
```

### Step 2: Set Environment Variables

Add to `.env`:
```bash
VORTEX_API_KEY=your_api_key_here
```

Load in Rust:
```rust
// Add to Cargo.toml
[dependencies]
dotenvy = "0.15"

// In your code
use dotenvy::dotenv;

#[tokio::main]
async fn main() {
    dotenv().ok();
    let api_key = std::env::var("VORTEX_API_KEY")
        .expect("VORTEX_API_KEY must be set");
}
```

### Step 3: Create Vortex Client State

**Option A: Axum State**:
```rust
use axum::extract::State;
use std::sync::Arc;
use vortex_sdk::VortexClient;

#[derive(Clone)]
struct AppState {
    vortex: Arc<VortexClient>,
}

#[tokio::main]
async fn main() {
    let vortex = Arc::new(VortexClient::new(
        std::env::var("VORTEX_API_KEY").unwrap()
    ));

    let state = AppState { vortex };

    let app = axum::Router::new()
        .route("/api/vortex/jwt", axum::routing::post(generate_jwt))
        .with_state(state);

    // ... rest of setup
}
```

**Option B: Actix Web Data**:
```rust
use actix_web::{web, App, HttpServer};
use std::sync::Arc;
use vortex_sdk::VortexClient;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let vortex = Arc::new(VortexClient::new(
        std::env::var("VORTEX_API_KEY").unwrap()
    ));

    HttpServer::new(move || {
        App::new()
            .app_data(web::Data::new(vortex.clone()))
            .route("/api/vortex/jwt", web::post().to(generate_jwt))
    })
    .bind(("127.0.0.1", 8080))?
    .run()
    .await
}
```

**Option C: Rocket State**:
```rust
use rocket::{State, launch};
use std::sync::Arc;
use vortex_sdk::VortexClient;

struct VortexState {
    client: Arc<VortexClient>,
}

#[launch]
fn rocket() -> _ {
    let vortex = Arc::new(VortexClient::new(
        std::env::var("VORTEX_API_KEY").unwrap()
    ));

    rocket::build()
        .manage(VortexState { client: vortex })
        .mount("/api/vortex", routes![generate_jwt])
}
```

### Step 4: Define Request/Response Types

```rust
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
struct JwtRequest {
    #[serde(rename = "componentId")]
    component_id: Option<String>,
    scope: Option<String>,
    #[serde(rename = "scopeType")]
    scope_type: Option<String>,
}

#[derive(Debug, Serialize)]
struct JwtResponse {
    jwt: String,
}

#[derive(Debug, Serialize)]
struct ErrorResponse {
    error: String,
}

#[derive(Debug, Deserialize)]
struct AcceptInvitationsRequest {
    #[serde(rename = "invitationIds")]
    invitation_ids: Vec<String>,
    target: InvitationTargetData,
}

#[derive(Debug, Deserialize, Serialize)]
struct InvitationTargetData {
    #[serde(rename = "type")]
    target_type: String,
    value: String,
}

// User struct for authentication
#[derive(Debug, Clone)]
struct AuthUser {
    id: String,
    email: String,
    is_admin: bool,
}
```

### Step 5: Implement Authentication Extractor

**Axum Example**:
```rust
use axum::{
    async_trait,
    extract::{FromRequestParts, TypedHeader},
    headers::{authorization::Bearer, Authorization},
    http::{request::Parts, StatusCode},
    RequestPartsExt,
};

#[async_trait]
impl<S> FromRequestParts<S> for AuthUser
where
    S: Send + Sync,
{
    type Rejection = (StatusCode, &'static str);

    async fn from_request_parts(parts: &mut Parts, _state: &S) -> Result<Self, Self::Rejection> {
        // Extract JWT token from Authorization header
        let TypedHeader(Authorization(bearer)) = parts
            .extract::<TypedHeader<Authorization<Bearer>>>()
            .await
            .map_err(|_| (StatusCode::UNAUTHORIZED, "Missing authorization header"))?;

        // Decode and verify JWT (implement your JWT validation logic)
        let token = bearer.token();
        let user = verify_jwt(token)
            .map_err(|_| (StatusCode::UNAUTHORIZED, "Invalid token"))?;

        Ok(user)
    }
}

fn verify_jwt(token: &str) -> Result<AuthUser, Box<dyn std::error::Error>> {
    // Implement your JWT verification logic here
    // This is a placeholder - use jsonwebtoken crate or your auth system
    Ok(AuthUser {
        id: "user-123".to_string(),
        email: "user@example.com".to_string(),
        is_admin: false,
    })
}

fn to_vortex_user(user: &AuthUser) -> vortex_sdk::User {
    let mut vortex_user = vortex_sdk::User::new(&user.id, &user.email);
    if user.is_admin {
        vortex_user = vortex_user.with_admin_scopes(vec!["autojoin".to_string()]);
    }
    vortex_user
}
```

**Actix Web Example**:
```rust
use actix_web::{web, HttpRequest, HttpMessage, FromRequest, Error};
use futures::future::{ready, Ready};

impl FromRequest for AuthUser {
    type Error = Error;
    type Future = Ready<Result<Self, Self::Error>>;

    fn from_request(req: &HttpRequest, _: &mut actix_web::dev::Payload) -> Self::Future {
        // Extract token from Authorization header
        let auth_header = req.headers().get("Authorization");

        match auth_header {
            Some(header_value) => {
                let token = header_value
                    .to_str()
                    .unwrap_or("")
                    .strip_prefix("Bearer ")
                    .unwrap_or("");

                match verify_jwt(token) {
                    Ok(user) => ready(Ok(user)),
                    Err(_) => ready(Err(actix_web::error::ErrorUnauthorized("Invalid token"))),
                }
            }
            None => ready(Err(actix_web::error::ErrorUnauthorized("Missing authorization"))),
        }
    }
}
```

### Step 6: Implement JWT Generation Endpoint

**Axum Example**:
```rust
use axum::{
    extract::State,
    http::StatusCode,
    Json,
};
use std::collections::HashMap;

async fn generate_jwt(
    State(state): State<AppState>,
    auth_user: AuthUser,
    Json(request): Json<JwtRequest>,
) -> Result<Json<JwtResponse>, (StatusCode, Json<ErrorResponse>)> {
    let vortex_user = to_vortex_user(&auth_user);

    // Build extra properties
    let mut extra = HashMap::new();
    if let Some(component_id) = request.component_id {
        extra.insert("componentId".to_string(), serde_json::json!(component_id));
    }
    if let Some(scope) = request.scope {
        extra.insert("scope".to_string(), serde_json::json!(scope));
    }
    if let Some(scope_type) = request.scope_type {
        extra.insert("scopeType".to_string(), serde_json::json!(scope_type));
    }

    let extra_opt = if extra.is_empty() { None } else { Some(extra) };

    match state.vortex.generate_jwt(&vortex_user, extra_opt) {
        Ok(jwt) => Ok(Json(JwtResponse { jwt })),
        Err(e) => {
            eprintln!("JWT generation error: {:?}", e);
            Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ErrorResponse {
                    error: "Internal server error".to_string(),
                }),
            ))
        }
    }
}
```

**Actix Web Example**:
```rust
use actix_web::{web, HttpResponse, Responder};

async fn generate_jwt(
    vortex: web::Data<Arc<VortexClient>>,
    auth_user: AuthUser,
    request: web::Json<JwtRequest>,
) -> impl Responder {
    let vortex_user = to_vortex_user(&auth_user);

    let mut extra = HashMap::new();
    if let Some(ref component_id) = request.component_id {
        extra.insert("componentId".to_string(), serde_json::json!(component_id));
    }
    if let Some(ref scope) = request.scope {
        extra.insert("scope".to_string(), serde_json::json!(scope));
    }
    if let Some(ref scope_type) = request.scope_type {
        extra.insert("scopeType".to_string(), serde_json::json!(scope_type));
    }

    let extra_opt = if extra.is_empty() { None } else { Some(extra) };

    match vortex.generate_jwt(&vortex_user, extra_opt) {
        Ok(jwt) => HttpResponse::Ok().json(JwtResponse { jwt }),
        Err(e) => {
            eprintln!("JWT generation error: {:?}", e);
            HttpResponse::InternalServerError().json(ErrorResponse {
                error: "Internal server error".to_string(),
            })
        }
    }
}
```

**Rocket Example**:
```rust
use rocket::{State, serde::json::Json};

#[post("/jwt", data = "<request>")]
async fn generate_jwt(
    vortex_state: &State<VortexState>,
    auth_user: AuthUser,
    request: Json<JwtRequest>,
) -> Result<Json<JwtResponse>, Status> {
    let vortex_user = to_vortex_user(&auth_user);

    let mut extra = HashMap::new();
    if let Some(ref component_id) = request.component_id {
        extra.insert("componentId".to_string(), serde_json::json!(component_id));
    }

    let extra_opt = if extra.is_empty() { None } else { Some(extra) };

    match vortex_state.client.generate_jwt(&vortex_user, extra_opt) {
        Ok(jwt) => Ok(Json(JwtResponse { jwt })),
        Err(_) => Err(Status::InternalServerError),
    }
}
```

### Step 7: Implement Invitation Query Endpoints

**Axum Example**:
```rust
use axum::extract::Query;

#[derive(Deserialize)]
struct InvitationQuery {
    #[serde(rename = "type")]
    target_type: String,
    value: String,
}

async fn get_invitations_by_target(
    State(state): State<AppState>,
    _auth_user: AuthUser,
    Query(query): Query<InvitationQuery>,
) -> Result<Json<serde_json::Value>, (StatusCode, Json<ErrorResponse>)> {
    match state.vortex
        .get_invitations_by_target(&query.target_type, &query.value)
        .await
    {
        Ok(invitations) => Ok(Json(serde_json::json!({ "invitations": invitations }))),
        Err(e) => {
            eprintln!("Get invitations error: {:?}", e);
            Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ErrorResponse {
                    error: "Internal server error".to_string(),
                }),
            ))
        }
    }
}

async fn get_invitation(
    State(state): State<AppState>,
    _auth_user: AuthUser,
    axum::extract::Path(invitation_id): axum::extract::Path<String>,
) -> Result<Json<vortex_sdk::Invitation>, (StatusCode, Json<ErrorResponse>)> {
    match state.vortex.get_invitation(&invitation_id).await {
        Ok(invitation) => Ok(Json(invitation)),
        Err(e) => {
            eprintln!("Get invitation error: {:?}", e);
            Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ErrorResponse {
                    error: "Internal server error".to_string(),
                }),
            ))
        }
    }
}

async fn get_invitations_by_group(
    State(state): State<AppState>,
    _auth_user: AuthUser,
    axum::extract::Path((group_type, group_id)): axum::extract::Path<(String, String)>,
) -> Result<Json<serde_json::Value>, (StatusCode, Json<ErrorResponse>)> {
    match state.vortex
        .get_invitations_by_group(&group_type, &group_id)
        .await
    {
        Ok(invitations) => Ok(Json(serde_json::json!({ "invitations": invitations }))),
        Err(e) => {
            eprintln!("Get invitations by group error: {:?}", e);
            Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ErrorResponse {
                    error: "Internal server error".to_string(),
                }),
            ))
        }
    }
}
```

### Step 8: Implement Accept Invitations Endpoint (CRITICAL)

**Axum Example with SQLx**:
```rust
use sqlx::{PgPool, Postgres};

// Add to AppState
#[derive(Clone)]
struct AppState {
    vortex: Arc<VortexClient>,
    db: PgPool,
}

async fn accept_invitations(
    State(state): State<AppState>,
    auth_user: AuthUser,
    Json(request): Json<AcceptInvitationsRequest>,
) -> Result<Json<serde_json::Value>, (StatusCode, Json<ErrorResponse>)> {
    // Step 1: Mark invitations as accepted in Vortex
    let target = vortex_sdk::InvitationTarget::new(
        &request.target.target_type,
        &request.target.value,
    );

    if let Err(e) = state.vortex
        .accept_invitations(request.invitation_ids.clone(), target)
        .await
    {
        eprintln!("Accept invitations error: {:?}", e);
        return Err((
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ErrorResponse {
                error: "Failed to accept invitations".to_string(),
            }),
        ));
    }

    // Step 2: CRITICAL - Add user to groups in YOUR database
    let mut tx = match state.db.begin().await {
        Ok(tx) => tx,
        Err(e) => {
            eprintln!("Transaction error: {:?}", e);
            return Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ErrorResponse {
                    error: "Database error".to_string(),
                }),
            ));
        }
    };

    for invitation_id in &request.invitation_ids {
        // Get invitation details to know what groups to add user to
        let invitation = match state.vortex.get_invitation(invitation_id).await {
            Ok(inv) => inv,
            Err(e) => {
                eprintln!("Get invitation error: {:?}", e);
                continue;
            }
        };

        // Add user to each group in the invitation
        for group in &invitation.groups {
            let query = sqlx::query!(
                r#"
                INSERT INTO group_memberships (user_id, group_type, group_id, role, joined_at)
                VALUES ($1, $2, $3, $4, NOW())
                ON CONFLICT (user_id, group_type, group_id)
                DO UPDATE SET role = EXCLUDED.role
                "#,
                auth_user.id,
                group.group_type,
                group.group_id,
                "member"
            );

            if let Err(e) = query.execute(&mut *tx).await {
                eprintln!("Database insert error: {:?}", e);
                let _ = tx.rollback().await;
                return Err((
                    StatusCode::INTERNAL_SERVER_ERROR,
                    Json(ErrorResponse {
                        error: "Database error".to_string(),
                    }),
                ));
            }
        }
    }

    if let Err(e) = tx.commit().await {
        eprintln!("Transaction commit error: {:?}", e);
        return Err((
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ErrorResponse {
                error: "Database error".to_string(),
            }),
        ));
    }

    Ok(Json(serde_json::json!({
        "success": true,
        "acceptedCount": request.invitation_ids.len()
    })))
}
```

**Diesel ORM Example**:
```rust
use diesel::prelude::*;
use diesel::r2d2::{self, ConnectionManager};

type DbPool = r2d2::Pool<ConnectionManager<PgConnection>>;

#[derive(Insertable)]
#[diesel(table_name = group_memberships)]
struct NewGroupMembership {
    user_id: String,
    group_type: String,
    group_id: String,
    role: String,
}

async fn accept_invitations_diesel(
    State(state): State<AppState>,
    auth_user: AuthUser,
    Json(request): Json<AcceptInvitationsRequest>,
) -> Result<Json<serde_json::Value>, (StatusCode, Json<ErrorResponse>)> {
    // Step 1: Accept in Vortex
    let target = vortex_sdk::InvitationTarget::new(
        &request.target.target_type,
        &request.target.value,
    );

    state.vortex
        .accept_invitations(request.invitation_ids.clone(), target)
        .await
        .map_err(|e| {
            eprintln!("Accept invitations error: {:?}", e);
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ErrorResponse {
                    error: "Failed to accept invitations".to_string(),
                }),
            )
        })?;

    // Step 2: Add to database
    let db = state.db.clone();
    let user_id = auth_user.id.clone();
    let invitation_ids = request.invitation_ids.clone();

    tokio::task::spawn_blocking(move || {
        let mut conn = db.get().map_err(|_| "Database connection error")?;

        conn.transaction::<_, diesel::result::Error, _>(|conn| {
            for invitation_id in &invitation_ids {
                // Fetch invitation (in real code, cache these)
                // Then insert memberships
                diesel::insert_into(group_memberships::table)
                    .values(&new_membership)
                    .on_conflict((
                        group_memberships::user_id,
                        group_memberships::group_type,
                        group_memberships::group_id,
                    ))
                    .do_update()
                    .set(group_memberships::role.eq("member"))
                    .execute(conn)?;
            }
            Ok(())
        })
    })
    .await
    .map_err(|_| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ErrorResponse {
                error: "Database error".to_string(),
            }),
        )
    })?
    .map_err(|_| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ErrorResponse {
                error: "Database error".to_string(),
            }),
        )
    })?;

    Ok(Json(serde_json::json!({
        "success": true,
        "acceptedCount": invitation_ids.len()
    })))
}
```

### Step 9: Implement Delete/Revoke Endpoints

**Axum Example**:
```rust
async fn revoke_invitation(
    State(state): State<AppState>,
    auth_user: AuthUser,
    axum::extract::Path(invitation_id): axum::extract::Path<String>,
) -> Result<Json<serde_json::Value>, (StatusCode, Json<ErrorResponse>)> {
    // Optional: Add authorization check
    // if !auth_user.is_admin {
    //     return Err((StatusCode::FORBIDDEN, Json(ErrorResponse { error: "Forbidden".to_string() })));
    // }

    match state.vortex.revoke_invitation(&invitation_id).await {
        Ok(_) => Ok(Json(serde_json::json!({ "success": true }))),
        Err(e) => {
            eprintln!("Revoke invitation error: {:?}", e);
            Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ErrorResponse {
                    error: "Internal server error".to_string(),
                }),
            ))
        }
    }
}

async fn delete_invitations_by_group(
    State(state): State<AppState>,
    auth_user: AuthUser,
    axum::extract::Path((group_type, group_id)): axum::extract::Path<(String, String)>,
) -> Result<Json<serde_json::Value>, (StatusCode, Json<ErrorResponse>)> {
    // Admin-only operation
    if !auth_user.is_admin {
        return Err((
            StatusCode::FORBIDDEN,
            Json(ErrorResponse {
                error: "Forbidden".to_string(),
            }),
        ));
    }

    match state.vortex
        .delete_invitations_by_group(&group_type, &group_id)
        .await
    {
        Ok(_) => Ok(Json(serde_json::json!({ "success": true }))),
        Err(e) => {
            eprintln!("Delete invitations by group error: {:?}", e);
            Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ErrorResponse {
                    error: "Internal server error".to_string(),
                }),
            ))
        }
    }
}

async fn reinvite(
    State(state): State<AppState>,
    _auth_user: AuthUser,
    axum::extract::Path(invitation_id): axum::extract::Path<String>,
) -> Result<Json<serde_json::Value>, (StatusCode, Json<ErrorResponse>)> {
    match state.vortex.reinvite(&invitation_id).await {
        Ok(_) => Ok(Json(serde_json::json!({ "success": true }))),
        Err(e) => {
            eprintln!("Reinvite error: {:?}", e);
            Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ErrorResponse {
                    error: "Internal server error".to_string(),
                }),
            ))
        }
    }
}
```

### Step 10: Database Schema

**SQL Migration** (using `sqlx-cli` or similar):
```sql
-- migrations/001_create_group_memberships.sql
CREATE TABLE IF NOT EXISTS group_memberships (
    id SERIAL PRIMARY KEY,
    user_id VARCHAR(255) NOT NULL,
    group_type VARCHAR(100) NOT NULL,
    group_id VARCHAR(255) NOT NULL,
    role VARCHAR(50) NOT NULL DEFAULT 'member',
    joined_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    CONSTRAINT unique_membership UNIQUE (user_id, group_type, group_id)
);

CREATE INDEX idx_group ON group_memberships (group_type, group_id);
CREATE INDEX idx_user ON group_memberships (user_id);
```

**Diesel Schema**:
```rust
// schema.rs
diesel::table! {
    group_memberships (id) {
        id -> Int4,
        user_id -> Varchar,
        group_type -> Varchar,
        group_id -> Varchar,
        role -> Varchar,
        joined_at -> Timestamp,
    }
}
```

### Step 11: Complete Axum App Example

```rust
use axum::{
    routing::{get, post, delete},
    Router,
};
use std::sync::Arc;
use tokio::net::TcpListener;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    dotenvy::dotenv().ok();

    let vortex = Arc::new(VortexClient::new(
        std::env::var("VORTEX_API_KEY")?
    ));

    let db = sqlx::PgPool::connect(&std::env::var("DATABASE_URL")?).await?;

    let state = AppState { vortex, db };

    let app = Router::new()
        .route("/api/vortex/jwt", post(generate_jwt))
        .route("/api/vortex/invitations", get(get_invitations_by_target))
        .route("/api/vortex/invitations/accept", post(accept_invitations))
        .route("/api/vortex/invitations/:invitation_id", get(get_invitation))
        .route("/api/vortex/invitations/:invitation_id", delete(revoke_invitation))
        .route("/api/vortex/invitations/:invitation_id/reinvite", post(reinvite))
        .route("/api/vortex/invitations/by-group/:group_type/:group_id", get(get_invitations_by_group))
        .route("/api/vortex/invitations/by-group/:group_type/:group_id", delete(delete_invitations_by_group))
        .with_state(state);

    let listener = TcpListener::bind("127.0.0.1:3000").await?;
    println!("Server running on http://127.0.0.1:3000");

    axum::serve(listener, app).await?;

    Ok(())
}
```

---

## Build and Validation

### Build Project
```bash
cargo build
```

### Run Tests
```bash
cargo test
```

### Check Code
```bash
cargo check
cargo clippy
```

### Format Code
```bash
cargo fmt
```

### Run Migrations
```bash
# SQLx
sqlx migrate run

# Diesel
diesel migration run
```

### Start Development Server
```bash
cargo run
```

### Test Endpoints

**1. Generate JWT**:
```bash
curl -X POST http://localhost:3000/api/vortex/jwt \
  -H "Authorization: Bearer your-auth-token" \
  -H "Content-Type: application/json" \
  -d '{}'
```

**2. Get Invitations by Target**:
```bash
curl -X GET "http://localhost:3000/api/vortex/invitations?type=email&value=user@example.com" \
  -H "Authorization: Bearer your-auth-token"
```

**3. Accept Invitations**:
```bash
curl -X POST http://localhost:3000/api/vortex/invitations/accept \
  -H "Authorization: Bearer your-auth-token" \
  -H "Content-Type: application/json" \
  -d '{
    "invitationIds": ["inv-123"],
    "target": {"type": "email", "value": "user@example.com"}
  }'
```

---

## Implementation Report

After implementing, provide this structured report:

```markdown
## Vortex Rust SDK Integration Report

### Files Created/Modified
- [ ] `Cargo.toml` - Added vortex-sdk dependency
- [ ] `src/main.rs` - Application entry point with state setup
- [ ] `src/handlers/vortex.rs` - Vortex endpoint handlers
- [ ] `src/auth.rs` - Authentication extractor
- [ ] `src/types.rs` - Request/response types
- [ ] `migrations/001_create_group_memberships.sql` - Database migration

### Framework Used
- [ ] Axum (version: ___)
- [ ] Actix Web (version: ___)
- [ ] Rocket (version: ___)
- [ ] Other: ___

### Endpoints Implemented
- [x] `POST /api/vortex/jwt` - JWT generation
- [x] `GET /api/vortex/invitations` - Get invitations by target
- [x] `POST /api/vortex/invitations/accept` - Accept invitations (with database logic)
- [x] `GET /api/vortex/invitations/:id` - Get single invitation
- [x] `DELETE /api/vortex/invitations/:id` - Revoke invitation
- [x] `POST /api/vortex/invitations/:id/reinvite` - Resend invitation
- [x] `GET /api/vortex/invitations/by-group/:type/:id` - Get invitations by group
- [x] `DELETE /api/vortex/invitations/by-group/:type/:id` - Delete invitations by group

### Database Integration
- [x] Created `group_memberships` table
- [x] Implemented database insert logic in accept endpoint
- [x] Added database indexes
- [x] Tested database inserts

### Configuration
- [x] Set `VORTEX_API_KEY` environment variable
- [x] Created VortexClient instance
- [x] Implemented authentication extraction
- [x] Set up error handling

### Testing Results
- [ ] JWT generation: ✓ Working
- [ ] Get invitations by target: ✓ Working
- [ ] Accept invitations: ✓ Working (database inserts confirmed)
- [ ] Get single invitation: ✓ Working
- [ ] Revoke invitation: ✓ Working
- [ ] Reinvite: ✓ Working
- [ ] Get invitations by group: ✓ Working
- [ ] Delete invitations by group: ✓ Working

### Notes
- Framework: [Axum / Actix Web / Rocket / other]
- Database: [PostgreSQL / MySQL / SQLite]
- ORM: [SQLx / Diesel / SeaORM / other]
- Authentication: [JWT / session / other]
```

---

## Common Issues and Solutions

### Issue 1: "error: failed to resolve: use of undeclared crate or module `vortex_sdk`"
**Solution**: Add to `Cargo.toml` and run `cargo build`:
```toml
[dependencies]
vortex-sdk = "1.0"
```

### Issue 2: "VORTEX_API_KEY environment variable not set"
**Solution**: Create `.env` file and load with `dotenvy`:
```toml
[dependencies]
dotenvy = "0.15"
```

```rust
dotenvy::dotenv().ok();
```

### Issue 3: Accept invitations succeeds but user not added to groups
**Solution**: Database logic not implemented. Check:
- Migration has been run
- Database connection is working
- Insert code is in accept endpoint
- Transaction is being committed

### Issue 4: "the trait bound `VortexClient: Clone` is not satisfied"
**Solution**: Wrap in `Arc`:
```rust
let vortex = Arc::new(VortexClient::new(api_key));
```

### Issue 5: Async runtime errors
**Solution**: Ensure tokio runtime is configured:
```toml
[dependencies]
tokio = { version = "1.0", features = ["full"] }
```

```rust
#[tokio::main]
async fn main() {
    // Your code
}
```

### Issue 6: CORS errors from frontend
**Solution**: Add tower-http CORS layer:

**Axum**:
```toml
[dependencies]
tower-http = { version = "0.5", features = ["cors"] }
```

```rust
use tower_http::cors::{CorsLayer, Any};

let app = Router::new()
    .route(/*...*/)
    .layer(
        CorsLayer::new()
            .allow_origin("http://localhost:3000".parse::<HeaderValue>().unwrap())
            .allow_methods([Method::GET, Method::POST, Method::DELETE])
            .allow_headers(Any)
    );
```

---

## Best Practices

### 1. Use `Arc` for Shared State
```rust
let vortex = Arc::new(VortexClient::new(api_key));
```

### 2. Error Handling with `Result`
```rust
async fn handler() -> Result<Json<T>, (StatusCode, Json<ErrorResponse>)> {
    // Handle errors explicitly
}
```

### 3. Use Strong Types
```rust
#[derive(Debug, Serialize, Deserialize)]
struct JwtRequest {
    #[serde(rename = "componentId")]
    component_id: Option<String>,
}
```

### 4. Database Transactions
```rust
let mut tx = db.begin().await?;
// Multiple operations
tx.commit().await?;
```

### 5. Environment Variables
```rust
use dotenvy::dotenv;

dotenv().ok();
let api_key = std::env::var("VORTEX_API_KEY")
    .expect("VORTEX_API_KEY must be set");
```

### 6. Logging
```rust
use tracing::{info, error};

error!("Accept invitations error: {:?}", e);
info!("Server running on {}", addr);
```

### 7. Testing
```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_generate_jwt() {
        let client = VortexClient::new("test-key".to_string());
        let user = User::new("test-123", "test@example.com");
        let jwt = client.generate_jwt(&user, None);
        assert!(jwt.is_ok());
    }
}
```

### 8. Use Builders
```rust
let user = User::new("user-123", "user@example.com")
    .with_admin_scopes(vec!["autojoin".to_string()]);
```

---

## Additional Resources

- **Rust SDK README**: `packages/vortex-rust-sdk/README.md`
- **Crates.io**: https://crates.io/crates/vortex-sdk
- **Axum Documentation**: https://docs.rs/axum/
- **Actix Web Documentation**: https://actix.rs/
- **Rocket Documentation**: https://rocket.rs/
- **SQLx Documentation**: https://docs.rs/sqlx/
- **Diesel Documentation**: https://diesel.rs/
