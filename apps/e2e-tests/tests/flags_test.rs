//! Flags E2E Tests

use e2e_tests::{unique_id, TestClient};
use flaglite_client::{CreateFlagRequest, FlagLiteError, FlagType};
use serial_test::serial;

/// Helper to get the first project ID for a test client
async fn get_project_id(client: &flaglite_client::FlagLiteClient) -> String {
    let projects = client.list_projects().await.expect("list_projects failed");
    assert!(!projects.is_empty(), "No projects found");
    projects[0].id.to_string()
}

/// Test creating a flag
#[tokio::test]
#[serial]
async fn test_create_flag() {
    let test_client = TestClient::signup().await.expect("Signup failed");
    let client = test_client.authenticated();
    let project_id = get_project_id(&client).await;

    let flag_key = format!("flag_{}", unique_id());
    let req = CreateFlagRequest {
        key: flag_key.clone(),
        name: "Test Flag".to_string(),
        description: Some("Created by e2e tests".to_string()),
        flag_type: FlagType::Boolean,
        enabled: true,
    };

    let result = client.create_flag(&project_id, req).await;
    assert!(result.is_ok(), "create_flag failed: {:?}", result.err());

    let flag = result.unwrap();
    assert_eq!(flag.key, flag_key);
    assert_eq!(flag.name, "Test Flag");
    assert_eq!(flag.flag_type, FlagType::Boolean);
}

/// Test listing flags
#[tokio::test]
#[serial]
async fn test_list_flags() {
    let test_client = TestClient::signup().await.expect("Signup failed");
    let client = test_client.authenticated();
    let project_id = get_project_id(&client).await;

    // Create a flag first
    let flag_key = format!("list_flag_{}", unique_id());
    let req = CreateFlagRequest {
        key: flag_key.clone(),
        name: "List Test Flag".to_string(),
        description: None,
        flag_type: FlagType::Boolean,
        enabled: false,
    };
    client
        .create_flag(&project_id, req)
        .await
        .expect("create_flag failed");

    // List flags
    let result = client.list_flags(&project_id, None).await;
    assert!(result.is_ok(), "list_flags failed: {:?}", result.err());

    let flags = result.unwrap();
    // Should have at least the flag we created
    let created_flag = flags.iter().find(|f| f.flag.key == flag_key);
    assert!(
        created_flag.is_some(),
        "Created flag not found in list. Flags: {:?}",
        flags.iter().map(|f| &f.flag.key).collect::<Vec<_>>()
    );
}

/// Test getting a specific flag
#[tokio::test]
#[serial]
async fn test_get_flag() {
    let test_client = TestClient::signup().await.expect("Signup failed");
    let client = test_client.authenticated();
    let project_id = get_project_id(&client).await;

    // Create a flag
    let flag_key = format!("get_flag_{}", unique_id());
    let req = CreateFlagRequest {
        key: flag_key.clone(),
        name: "Get Test Flag".to_string(),
        description: Some("Test description".to_string()),
        flag_type: FlagType::Boolean,
        enabled: true,
    };
    client
        .create_flag(&project_id, req)
        .await
        .expect("create_flag failed");

    // Get the flag
    let result = client.get_flag(&project_id, &flag_key, None).await;
    assert!(result.is_ok(), "get_flag failed: {:?}", result.err());

    let flag_with_state = result.unwrap();
    assert_eq!(flag_with_state.flag.key, flag_key);
    assert_eq!(flag_with_state.flag.name, "Get Test Flag");
}

/// Test toggling a flag
#[tokio::test]
#[serial]
async fn test_toggle_flag() {
    let test_client = TestClient::signup().await.expect("Signup failed");
    let client = test_client.authenticated();
    let project_id = get_project_id(&client).await;

    // Create a flag (enabled = false)
    let flag_key = format!("toggle_flag_{}", unique_id());
    let req = CreateFlagRequest {
        key: flag_key.clone(),
        name: "Toggle Test Flag".to_string(),
        description: None,
        flag_type: FlagType::Boolean,
        enabled: false,
    };
    client
        .create_flag(&project_id, req)
        .await
        .expect("create_flag failed");

    // Get initial state (in production environment)
    let initial = client
        .get_flag(&project_id, &flag_key, Some("production"))
        .await
        .expect("get_flag failed");
    let initial_enabled = initial.enabled;

    // Toggle the flag
    let result = client
        .toggle_flag(&project_id, &flag_key, "production")
        .await;
    assert!(result.is_ok(), "toggle_flag failed: {:?}", result.err());

    let toggled = result.unwrap();
    assert_ne!(
        toggled.enabled, initial_enabled,
        "Flag enabled state should have changed"
    );

    // Toggle again to verify it goes back
    let toggled_back = client
        .toggle_flag(&project_id, &flag_key, "production")
        .await
        .expect("toggle_flag failed");
    assert_eq!(
        toggled_back.enabled, initial_enabled,
        "Flag should return to initial state after double toggle"
    );
}

/// Test getting a non-existent flag returns error
#[tokio::test]
#[serial]
async fn test_get_nonexistent_flag() {
    let test_client = TestClient::signup().await.expect("Signup failed");
    let client = test_client.authenticated();
    let project_id = get_project_id(&client).await;

    let result = client
        .get_flag(&project_id, "nonexistent_flag_key", None)
        .await;
    assert!(result.is_err(), "get_flag should fail for nonexistent flag");

    match result.unwrap_err() {
        FlagLiteError::FlagNotFound(_) => {}
        FlagLiteError::ApiError { status: 404, .. } => {}
        other => panic!("Expected FlagNotFound or 404 error, got: {other:?}"),
    }
}

/// Test creating flag with different types
#[tokio::test]
#[serial]
async fn test_create_flag_types() {
    let test_client = TestClient::signup().await.expect("Signup failed");
    let client = test_client.authenticated();
    let project_id = get_project_id(&client).await;

    let types = [
        (FlagType::Boolean, "bool"),
        (FlagType::String, "str"),
        (FlagType::Number, "num"),
        (FlagType::Json, "json"),
    ];

    for (flag_type, suffix) in types {
        let flag_key = format!("typed_flag_{}_{}", suffix, unique_id());
        let req = CreateFlagRequest {
            key: flag_key.clone(),
            name: format!("{flag_type:?} Flag"),
            description: None,
            flag_type,
            enabled: false,
        };

        let result = client.create_flag(&project_id, req).await;
        assert!(
            result.is_ok(),
            "create_flag for {:?} failed: {:?}",
            flag_type,
            result.err()
        );

        let flag = result.unwrap();
        assert_eq!(flag.flag_type, flag_type);
    }
}

/// Test listing flags with environment filter
#[tokio::test]
#[serial]
async fn test_list_flags_with_environment() {
    let test_client = TestClient::signup().await.expect("Signup failed");
    let client = test_client.authenticated();
    let project_id = get_project_id(&client).await;

    // Create a flag
    let flag_key = format!("env_flag_{}", unique_id());
    let req = CreateFlagRequest {
        key: flag_key.clone(),
        name: "Env Test Flag".to_string(),
        description: None,
        flag_type: FlagType::Boolean,
        enabled: true,
    };
    client
        .create_flag(&project_id, req)
        .await
        .expect("create_flag failed");

    // List with production environment filter
    let result = client.list_flags(&project_id, Some("production")).await;
    assert!(
        result.is_ok(),
        "list_flags with env failed: {:?}",
        result.err()
    );

    let flags = result.unwrap();
    let created_flag = flags.iter().find(|f| f.flag.key == flag_key);
    assert!(
        created_flag.is_some(),
        "Flag should appear in production environment"
    );
}
