use std::collections::HashMap;
use vortex_sdk::{User, VortexClient};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize the Vortex client
    let api_key = std::env::var("VORTEX_API_KEY")
        .unwrap_or_else(|_| "demo-api-key".to_string());

    let client = VortexClient::new(api_key);

    // Example 1: Generate JWT - simple usage
    println!("=== JWT Generation Example ===");
    let user = User::new("user-123", "user@example.com")
        .with_admin_scopes(vec!["autojoin".to_string()]);

    let jwt1 = client.generate_jwt(&user, None)?;
    println!("Generated JWT: {}\n", jwt1);

    // Example 2: Generate JWT with additional properties
    println!("=== JWT Generation with Additional Properties ===");
    let user2 = User::new("user-456", "user@example.com");

    let mut extra = HashMap::new();
    extra.insert("role".to_string(), serde_json::json!("admin"));
    extra.insert("department".to_string(), serde_json::json!("Engineering"));

    let jwt2 = client.generate_jwt(&user2, Some(extra))?;
    println!("Generated JWT with extra: {}\n", jwt2);

    // Example 3: Get invitations by target
    println!("=== Get Invitations by Target Example ===");
    match client
        .get_invitations_by_target("email", "user@example.com")
        .await
    {
        Ok(invitations) => {
            println!("Found {} invitations", invitations.len());
        }
        Err(e) => {
            eprintln!("Error: {}", e);
        }
    }

    println!("\n=== Example Complete ===");
    println!("To use with real data, set VORTEX_API_KEY environment variable");

    Ok(())
}
