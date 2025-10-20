use vortex_sdk::{VortexClient, Identifier, Group};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize the Vortex client
    let api_key = std::env::var("VORTEX_API_KEY")
        .expect("Please set VORTEX_API_KEY environment variable");

    let client = VortexClient::new(api_key);

    // Example user data
    let user_id = "user-123";
    let identifiers = vec![
        Identifier::new("email", "user@example.com"),
        Identifier::new("sms", "18008675309"),
    ];
    let groups = vec![
        Group::new("workspace", "ws-1", "Main Workspace"),
        Group::new("team", "team-1", "Engineering"),
    ];
    let role = Some("admin");

    // Generate a JWT
    println!("Generating JWT...");
    let jwt = client.generate_jwt(user_id, identifiers, groups, role)?;
    println!("JWT: {}\n", jwt);

    // Example: Get invitations by target
    println!("Fetching invitations by email...");
    match client.get_invitations_by_target("email", "user@example.com").await {
        Ok(invitations) => {
            println!("Found {} invitations", invitations.len());
        }
        Err(e) => {
            eprintln!("Error: {}", e);
        }
    }

    Ok(())
}
