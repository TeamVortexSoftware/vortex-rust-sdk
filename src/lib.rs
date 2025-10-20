//! Vortex Rust SDK
//!
//! This crate provides a Rust SDK for the Vortex authentication and invitation management platform.
//!
//! # Features
//!
//! - Generate JWTs compatible with React providers
//! - Full API integration for invitation management
//! - Async/await support with tokio
//! - Type-safe API with comprehensive error handling
//!
//! # Example
//!
//! ```no_run
//! use vortex_sdk::{VortexClient, Identifier, Group};
//!
//! #[tokio::main]
//! async fn main() {
//!     let client = VortexClient::new(std::env::var("VORTEX_API_KEY").unwrap());
//!
//!     // Generate a JWT
//!     let jwt = client.generate_jwt(
//!         "user-123",
//!         vec![Identifier::new("email", "user@example.com")],
//!         vec![Group::new("team", "team-1", "Engineering")],
//!         Some("admin")
//!     ).unwrap();
//!
//!     println!("JWT: {}", jwt);
//!
//!     // Get invitations
//!     let invitations = client
//!         .get_invitations_by_target("email", "user@example.com")
//!         .await
//!         .unwrap();
//!
//!     println!("Found {} invitations", invitations.len());
//! }
//! ```

mod client;
mod error;
mod types;

pub use client::VortexClient;
pub use error::VortexError;
pub use types::*;
