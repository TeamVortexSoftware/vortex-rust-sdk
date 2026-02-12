# Vortex Rust SDK

This crate provides the Vortex Rust SDK for authentication and invitation management.

With this SDK, you can generate JWTs for use with the Vortex Widget and make API calls to the Vortex API.

## Features

### Invitation Delivery Types

Vortex supports multiple delivery methods for invitations:

- **`email`** - Email invitations sent by Vortex (includes reminders and nudges)
- **`phone`** - Phone invitations sent by the user/customer
- **`share`** - Shareable invitation links for social sharing
- **`internal`** - Internal invitations managed entirely by your application
  - No email/SMS communication triggered by Vortex
  - Target value can be any customer-defined identifier (UUID, string, number)
  - Useful for in-app invitation flows where you handle the delivery
  - Example use case: In-app notifications, dashboard invites, etc.

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
use vortex_sdk::{VortexClient, User};

fn main() {
    // Initialize the Vortex client with your API key
    let client = VortexClient::new(std::env::var("VORTEX_API_KEY").unwrap());

    // Create a user and generate JWT
    let user = User::builder()
        .id("user-123")
        .email("user@example.com")
        .user_name("Jane Doe")                                          // Optional: user's display name
        .user_avatar_url("https://example.com/avatars/jane.jpg")        // Optional: user's avatar URL
        .admin_scopes(vec!["autojoin".to_string()])                // Optional: grants autojoin admin privileges
        .build();

    let jwt = client.generate_jwt(&user, None).unwrap();

    println!("JWT: {}", jwt);
}
```

### Async API Usage

All API methods are async and require a tokio runtime:

```rust
use vortex_sdk::{VortexClient, User};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let client = VortexClient::new(std::env::var("VORTEX_API_KEY")?);

    // Generate a JWT
    let user = User::new("user-123", "user@example.com")
        .with_admin_scopes(vec!["autojoin".to_string()]);
    let jwt = client.generate_jwt(&user, None)?;

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
use vortex_sdk::VortexClient;

#[derive(Clone)]
struct AppState {
    vortex: Arc<VortexClient>,
}

#[derive(Serialize)]
struct JwtResponse {
    jwt: String,
}

async fn get_jwt(State(state): State<AppState>) -> Result<Json<JwtResponse>, StatusCode> {
    let user = vortex_sdk::User::new("user-123", "user@example.com");
    let jwt = state
        .vortex
        .generate_jwt(&user, None)
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

#### Accept an Invitation

```rust
use vortex_sdk::AcceptUser;

let user = AcceptUser::new().with_email("user@example.com");
let result = client
    .accept_invitation("invitation-id", user)
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
- **Flexible**: User-based JWT generation with support for admin scopes and custom properties

## License

MIT

## Support

For support, please contact support@vortexsoftware.com or visit our [documentation](https://docs.vortexsoftware.com).
