# Vortex Rust SDK

This crate provides the Vortex Rust SDK for authentication and invitation management.

With this SDK, you can generate JWTs for use with the Vortex Widget and make API calls to the Vortex API.

## Installation

Add the SDK to your `Cargo.toml`:

```toml
[dependencies]
vortex-sdk = "1.0"
tokio = { version = "1.0", features = ["full"] }
```

## Getting Started

Once you have the SDK installed, [login](https://admin.vortexsoftware.com/signin) to Vortex and [create an API Key](https://admin.vortexsoftware.com/members/api-keys). Keep your API key safe! Vortex does not store the API key and it is not retrievable once it has been created.

Your API key is used to:
- Sign JWTs for use with the Vortex Widget
- Make API calls against the [Vortex API](https://api.vortexsoftware.com/api)

## Usage

### Generate a JWT for the Vortex Widget

The Vortex Widget requires a JWT to authenticate users. Here's how to generate one:

```rust
use vortex_sdk::{VortexClient, Identifier, Group};

fn main() {
    // Initialize the Vortex client with your API key
    let client = VortexClient::new(std::env::var("VORTEX_API_KEY").unwrap());

    // User ID from your system
    let user_id = "users-id-in-my-system";

    // Identifiers associated with the user
    let identifiers = vec![
        Identifier::new("email", "user@example.com"),
        Identifier::new("sms", "18008675309"),
    ];

    // Groups the user belongs to (specific to your product)
    let groups = vec![
        Group::new("workspace", "workspace-123", "My Workspace"),
        Group::new("document", "doc-456", "Project Plan"),
    ];

    // User role (if applicable)
    let role = Some("admin");

    // Generate the JWT
    let jwt = client.generate_jwt(user_id, identifiers, groups, role).unwrap();

    println!("JWT: {}", jwt);
}
```

### Async API Usage

All API methods are async and require a tokio runtime:

```rust
use vortex_sdk::{VortexClient, Identifier, Group, InvitationTarget};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let client = VortexClient::new(std::env::var("VORTEX_API_KEY")?);

    // Generate a JWT
    let jwt = client.generate_jwt(
        "user-123",
        vec![Identifier::new("email", "user@example.com")],
        vec![Group::new("team", "team-1", "Engineering")],
        Some("admin")
    )?;

    println!("JWT: {}", jwt);

    // Get invitations by target
    let invitations = client
        .get_invitations_by_target("email", "user@example.com")
        .await?;

    println!("Found {} invitations", invitations.len());

    Ok(())
}
```

### Using with Axum (Web Framework)

Here's an example of using the SDK with the Axum web framework:

```rust
use axum::{
    extract::State,
    http::StatusCode,
    response::Json,
    routing::get,
    Router,
};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use vortex_sdk::{VortexClient, Identifier, Group};

#[derive(Clone)]
struct AppState {
    vortex: Arc<VortexClient>,
}

#[derive(Serialize)]
struct JwtResponse {
    jwt: String,
}

async fn get_jwt(State(state): State<AppState>) -> Result<Json<JwtResponse>, StatusCode> {
    let jwt = state
        .vortex
        .generate_jwt(
            "user-123",
            vec![Identifier::new("email", "user@example.com")],
            vec![Group::new("workspace", "ws-1", "Main Workspace")],
            Some("member"),
        )
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(Json(JwtResponse { jwt }))
}

#[tokio::main]
async fn main() {
    let vortex = Arc::new(VortexClient::new(
        std::env::var("VORTEX_API_KEY").unwrap(),
    ));

    let app = Router::new()
        .route("/api/vortex-jwt", get(get_jwt))
        .with_state(AppState { vortex });

    let listener = tokio::net::TcpListener::bind("127.0.0.1:3000")
        .await
        .unwrap();

    axum::serve(listener, app).await.unwrap();
}
```

## API Methods

All API methods are asynchronous and require the tokio runtime.

### Invitation Management

#### Get Invitations by Target

```rust
let invitations = client
    .get_invitations_by_target("email", "user@example.com")
    .await?;
```

#### Get Invitation by ID

```rust
let invitation = client.get_invitation("invitation-id").await?;
```

#### Revoke Invitation

```rust
client.revoke_invitation("invitation-id").await?;
```

#### Accept Invitations

```rust
use vortex_sdk::InvitationTarget;

let target = InvitationTarget::new("email", "user@example.com");
let result = client
    .accept_invitations(
        vec!["invitation-id-1".to_string(), "invitation-id-2".to_string()],
        target,
    )
    .await?;
```

#### Get Invitations by Group

```rust
let invitations = client
    .get_invitations_by_group("workspace", "workspace-123")
    .await?;
```

#### Delete Invitations by Group

```rust
client
    .delete_invitations_by_group("workspace", "workspace-123")
    .await?;
```

#### Reinvite

```rust
let result = client.reinvite("invitation-id").await?;
```

## Error Handling

The SDK uses a custom `VortexError` type for error handling:

```rust
use vortex_sdk::{VortexClient, VortexError};

match client.get_invitation("invalid-id").await {
    Ok(invitation) => println!("Got invitation: {:?}", invitation),
    Err(VortexError::ApiError(msg)) => eprintln!("API error: {}", msg),
    Err(VortexError::HttpError(msg)) => eprintln!("HTTP error: {}", msg),
    Err(e) => eprintln!("Other error: {}", e),
}
```

## Requirements

- Rust 1.70 or higher
- Tokio runtime for async operations

## Features

- **Type-safe**: Full type safety with Rust's type system
- **Async/await**: Built on tokio for efficient async operations
- **React Compatible**: JWTs generated using the same algorithm as Node.js SDK
- **Comprehensive**: All Vortex API endpoints supported
- **Error handling**: Rich error types for better debugging

## License

MIT

## Support

For support, please contact support@vortexsoftware.com or visit our [documentation](https://docs.vortexsoftware.com).
