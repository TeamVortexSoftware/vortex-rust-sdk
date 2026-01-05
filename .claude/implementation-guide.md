# Vortex Rust SDK Implementation Guide

**Crate:** `vortex-sdk`
**Type:** Base SDK (Core library for Rust applications)
**Requires:** Rust edition 2021

## Prerequisites
From integration contract you need: API endpoint prefix, scope entity, authentication pattern
From discovery data you need: Rust framework (Axum, Actix Web, Rocket), database ORM, async pattern

## Key Facts
- Framework-agnostic Rust SDK
- Client-based: instantiate `VortexClient` struct
- Async/await with tokio
- Type-safe with full Rust type system
- Accept invitations requires custom database logic (must implement)
- Wrap client in `Arc` for shared state

---

## Step 1: Add Dependencies

Add to `Cargo.toml`:
```toml
[dependencies]
vortex-sdk = "1.0"
tokio = { version = "1.0", features = ["full"] }
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"
dotenvy = "0.15"

# Choose your web framework
axum = "0.7"  # For Axum
# OR
actix-web = "4.0"  # For Actix Web
# OR
rocket = "0.5"  # For Rocket

# Choose your database
sqlx = { version = "0.7", features = ["runtime-tokio-native-tls", "postgres"] }  # For SQLx
# OR
diesel = { version = "2.0", features = ["postgres", "r2d2"] }  # For Diesel
```

Then build:
```bash
cargo build
```

---

## Step 2: Set Environment Variable

Add to `.env`:

```bash
VORTEX_API_KEY=VRTX.your-api-key-here.secret
```

Load in code:
```rust
use dotenvy::dotenv;

#[tokio::main]
async fn main() {
    dotenv().ok();
    let api_key = std::env::var("VORTEX_API_KEY")
        .expect("VORTEX_API_KEY must be set");
}
```

**Never commit API key to version control.**

---

## Step 3: Create Vortex Client State

### Axum:
```rust
use axum::extract::State;
use std::sync::Arc;
use vortex_sdk::VortexClient;

#[derive(Clone)]
struct AppState {
    vortex: Arc<VortexClient>,
    db: sqlx::PgPool,  // Your database
}

#[tokio::main]
async fn main() {
    dotenv().ok();

    let vortex = Arc::new(VortexClient::new(
        std::env::var("VORTEX_API_KEY").unwrap()
    ));

    let db = sqlx::PgPool::connect(&std::env::var("DATABASE_URL").unwrap()).await.unwrap();

    let state = AppState { vortex, db };

    let app = axum::Router::new()
        .route("/api/vortex/jwt", axum::routing::post(generate_jwt))
        .with_state(state);

    let listener = tokio::net::TcpListener::bind("127.0.0.1:3000").await.unwrap();
    axum::serve(listener, app).await.unwrap();
}
```

### Actix Web:
```rust
use actix_web::{web, App, HttpServer};
use std::sync::Arc;
use vortex_sdk::VortexClient;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    dotenv().ok();

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

---

## Step 4: Define Types

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
    user: AcceptUser,
}

#[derive(Debug, Deserialize, Serialize)]
struct AcceptUser {
    email: Option<String>,
    phone: Option<String>,
}

#[derive(Debug, Clone)]
struct AuthUser {
    id: String,
    email: String,
    is_admin: bool,
}
```

---

## Step 5: Implement Authentication Extractor

### Axum:
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
        let TypedHeader(Authorization(bearer)) = parts
            .extract::<TypedHeader<Authorization<Bearer>>>()
            .await
            .map_err(|_| (StatusCode::UNAUTHORIZED, "Missing authorization header"))?;

        let token = bearer.token();
        let user = verify_jwt(token)
            .map_err(|_| (StatusCode::UNAUTHORIZED, "Invalid token"))?;

        Ok(user)
    }
}

fn verify_jwt(token: &str) -> Result<AuthUser, Box<dyn std::error::Error>> {
    // Implement your JWT verification logic
    // Use jsonwebtoken crate or your auth system
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

### Actix Web:
```rust
use actix_web::{web, HttpRequest, HttpMessage, FromRequest, Error};
use futures::future::{ready, Ready};

impl FromRequest for AuthUser {
    type Error = Error;
    type Future = Ready<Result<Self, Self::Error>>;

    fn from_request(req: &HttpRequest, _: &mut actix_web::dev::Payload) -> Self::Future {
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

**Adapt to their patterns:**
- Match their auth mechanism (JWT, custom)
- Match their user structure
- Match their admin detection logic

---

## Step 6: Implement JWT Generation Endpoint

### Axum:
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

### Actix Web:
```rust
use actix_web::{web, HttpResponse, Responder};
use std::collections::HashMap;

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

---

## Step 7: Implement Accept Invitations Endpoint (CRITICAL)

### Axum with SQLx:
```rust
use sqlx::PgPool;

async fn accept_invitations(
    State(state): State<AppState>,
    auth_user: AuthUser,
    Json(request): Json<AcceptInvitationsRequest>,
) -> Result<Json<serde_json::Value>, (StatusCode, Json<ErrorResponse>)> {
    // 1. Mark as accepted in Vortex
    if let Err(e) = state.vortex
        .accept_invitations(request.invitation_ids.clone(), &request.user)
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

    // 2. CRITICAL - Add to database
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
        let invitation = match state.vortex.get_invitation(invitation_id).await {
            Ok(inv) => inv,
            Err(e) => {
                eprintln!("Get invitation error: {:?}", e);
                continue;
            }
        };

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

**Critical - Adapt database logic:**
- Use their actual table names (from discovery)
- Use their actual field names
- Use their database library (SQLx, Diesel, SeaORM)
- Handle duplicate memberships if needed

---

## Step 8: Database Schema

### SQLx Migration:
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

### Diesel Schema:
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

---

## Step 9: Complete Axum App

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
        .with_state(state);

    let listener = TcpListener::bind("127.0.0.1:3000").await?;
    println!("Server running on http://127.0.0.1:3000");

    axum::serve(listener, app).await?;

    Ok(())
}
```

---

## Step 10: Build and Test

```bash
# Run migrations
sqlx migrate run  # SQLx
diesel migration run  # Diesel

# Build
cargo build

# Start server
cargo run

# Test JWT endpoint
curl -X POST http://localhost:3000/api/vortex/jwt \
  -H "Authorization: Bearer your-auth-token" \
  -H "Content-Type: application/json" \
  -d '{}'
```

Expected response:
```json
{
  "jwt": "eyJhbGciOiJIUzI1NiIs..."
}
```

---

## Common Errors

**"use of undeclared crate or module `vortex_sdk`"** → Add to `Cargo.toml` and run `cargo build`

**"VORTEX_API_KEY not set"** → Create `.env` file and use `dotenvy::dotenv().ok()`

**User not added to database** → Must implement database logic in accept handler (see Step 7)

**"the trait bound `VortexClient: Clone` is not satisfied"** → Wrap in `Arc`:
```rust
let vortex = Arc::new(VortexClient::new(api_key));
```

**Async runtime errors** → Ensure tokio configured:
```rust
#[tokio::main]
async fn main() {
    // Your code
}
```

**CORS errors** → Add tower-http CORS:
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

## After Implementation Report

List files created/modified:
- Dependency: Cargo.toml
- Entry: src/main.rs
- Handlers: src/handlers/vortex.rs
- Auth: src/auth.rs
- Types: src/types.rs
- Migration: migrations/001_create_group_memberships.sql

Confirm:
- Vortex SDK added to Cargo.toml
- VortexClient instance created and wrapped in Arc
- JWT endpoint returns valid JWT
- Accept invitations includes database logic
- Routes registered at correct prefix
- Migrations run

## Endpoints Registered

All endpoints at `/api/vortex`:
- `POST /jwt` - Generate JWT for authenticated user
- `GET /invitations` - Get invitations by target
- `GET /invitations/:id` - Get invitation by ID
- `POST /invitations/accept` - Accept invitations (custom DB logic)
- `DELETE /invitations/:id` - Revoke invitation
