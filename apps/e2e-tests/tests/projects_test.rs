//! Projects E2E Tests

use e2e_tests::{unique_id, TestClient};
use flaglite_client::CreateProjectRequest;
use serial_test::serial;

/// Test listing projects returns at least one project (created on signup)
#[tokio::test]
#[serial]
async fn test_list_projects() {
    let test_client = TestClient::signup().await.expect("Signup failed");
    let client = test_client.authenticated();

    let result = client.list_projects().await;
    assert!(result.is_ok(), "list_projects failed: {:?}", result.err());

    let projects = result.unwrap();
    // User should have at least one default project
    assert!(
        !projects.is_empty(),
        "Expected at least one project for new user"
    );
}

/// Test creating a new project
#[tokio::test]
#[serial]
async fn test_create_project() {
    let test_client = TestClient::signup().await.expect("Signup failed");
    let client = test_client.authenticated();

    let project_name = format!("Test Project {}", unique_id());
    let req = CreateProjectRequest {
        name: project_name.clone(),
        description: Some("Test project created by e2e tests".to_string()),
    };

    let result = client.create_project(req).await;
    assert!(result.is_ok(), "create_project failed: {:?}", result.err());

    let project = result.unwrap();
    assert_eq!(project.name, project_name);
    assert!(!project.slug.is_empty());
}

/// Test project has default environments
#[tokio::test]
#[serial]
async fn test_project_has_default_environments() {
    let test_client = TestClient::signup().await.expect("Signup failed");
    let client = test_client.authenticated();

    // Get the first project
    let projects = client.list_projects().await.expect("list_projects failed");
    assert!(!projects.is_empty(), "No projects found");

    let project_id = projects[0].id.to_string();

    // List environments
    let result = client.list_environments(&project_id).await;
    assert!(
        result.is_ok(),
        "list_environments failed: {:?}",
        result.err()
    );

    let environments = result.unwrap();

    // Should have default environments (typically: development, staging, production)
    assert!(
        !environments.is_empty(),
        "Expected default environments to be created"
    );

    // Check for production environment
    let has_production = environments
        .iter()
        .any(|e| e.name == "production" || e.slug == "production");
    assert!(
        has_production,
        "Expected a 'production' environment. Found: {:?}",
        environments.iter().map(|e| &e.name).collect::<Vec<_>>()
    );
}

/// Test creating multiple projects
#[tokio::test]
#[serial]
async fn test_create_multiple_projects() {
    let test_client = TestClient::signup().await.expect("Signup failed");
    let client = test_client.authenticated();

    // Get initial project count
    let initial_projects = client.list_projects().await.expect("list_projects failed");
    let initial_count = initial_projects.len();

    // Create two new projects
    for i in 0..2 {
        let req = CreateProjectRequest {
            name: format!("Multi Project {} {}", unique_id(), i),
            description: None,
        };
        client
            .create_project(req)
            .await
            .expect("create_project failed");
    }

    // Verify project count increased
    let final_projects = client.list_projects().await.expect("list_projects failed");

    assert_eq!(
        final_projects.len(),
        initial_count + 2,
        "Expected 2 new projects"
    );
}
