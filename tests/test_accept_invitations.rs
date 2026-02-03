use vortex_sdk::{AcceptUser, InvitationTarget, InvitationTargetType, VortexClient};

#[tokio::test]
async fn test_accept_user_with_email_only() {
    let user = AcceptUser::new().with_email("email@example.com");
    assert_eq!(user.email, Some("email@example.com".to_string()));
    assert_eq!(user.phone, None);
    assert_eq!(user.name, None);
}

#[tokio::test]
async fn test_accept_user_with_email_and_name() {
    let user = AcceptUser::new()
        .with_email("user@example.com")
        .with_name("John Doe");
    assert_eq!(user.email, Some("user@example.com".to_string()));
    assert_eq!(user.name, Some("John Doe".to_string()));
    assert_eq!(user.phone, None);
}

#[tokio::test]
async fn test_accept_user_with_all_fields() {
    let user = AcceptUser::new()
        .with_email("test@example.com")
        .with_phone("+1234567890")
        .with_name("Test User");
    assert_eq!(user.email, Some("test@example.com".to_string()));
    assert_eq!(user.phone, Some("+1234567890".to_string()));
    assert_eq!(user.name, Some("Test User".to_string()));
}

#[tokio::test]
async fn test_accept_invitations_with_user_fails_with_fake_key() {
    let client = VortexClient::new("VRTX.dGVzdC1rZXk.test-secret".to_string());
    let user = AcceptUser::new().with_email("email@example.com");

    let result = client.accept_invitations(vec!["test-inv".to_string()], user).await;
    assert!(result.is_err(), "Should fail with fake API key");
}

#[tokio::test]
async fn test_accept_invitations_with_legacy_target_fails_with_fake_key() {
    let client = VortexClient::new("VRTX.dGVzdC1rZXk.test-secret".to_string());
    let target = InvitationTarget::email("legacy@example.com");

    let result = client.accept_invitations(vec!["test-inv".to_string()], target).await;
    assert!(result.is_err(), "Should fail with fake API key");
}

#[tokio::test]
async fn test_accept_invitations_with_legacy_targets_array_fails_with_fake_key() {
    let client = VortexClient::new("VRTX.dGVzdC1rZXk.test-secret".to_string());
    let targets = vec![
        InvitationTarget::email("user1@example.com"),
        InvitationTarget::email("user2@example.com"),
    ];

    let result = client.accept_invitations(vec!["test-inv".to_string()], targets).await;
    assert!(result.is_err(), "Should fail with fake API key");
}

#[tokio::test]
async fn test_validation_user_without_email_or_phone() {
    let client = VortexClient::new("VRTX.dGVzdC1rZXk.test-secret".to_string());
    let invalid_user = AcceptUser::new().with_name("Just a Name");

    let result = client.accept_invitations(vec!["test-inv".to_string()], invalid_user).await;
    assert!(result.is_err(), "Should fail validation");

    let err_msg = format!("{:?}", result.err().unwrap());
    assert!(err_msg.contains("email or phone"), "Error should mention email or phone requirement");
}

#[tokio::test]
async fn test_validation_empty_targets_array() {
    let client = VortexClient::new("VRTX.dGVzdC1rZXk.test-secret".to_string());
    let empty_targets: Vec<InvitationTarget> = vec![];

    let result = client.accept_invitations(vec!["test-inv".to_string()], empty_targets).await;
    assert!(result.is_err(), "Should fail with empty targets");

    let err_msg = format!("{:?}", result.err().unwrap());
    assert!(err_msg.contains("No targets provided"), "Error should mention no targets provided");
}
