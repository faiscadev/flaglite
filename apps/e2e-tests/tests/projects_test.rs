//! Projects E2E Tests (Black-Box)
//!
//! Tests project management by:
//! - Spawning actual flaglite-api server
//! - Running actual flaglite CLI commands
//! - Parsing command output to verify results

mod common;

use common::{unique_project_name, TestHarness, TEST_PASSWORD};

/// Test listing projects returns at least one project (created on signup).
#[tokio::test]
async fn test_list_projects() {
    let harness = TestHarness::new("list_projects")
        .await
        .expect("Failed to create test harness");

    let user = harness.create_user("alice");

    // Sign up (should create default project)
    let signup_result = user.signup(None, TEST_PASSWORD);
    assert!(
        signup_result.is_ok(),
        "Signup failed: {:?}",
        signup_result.err()
    );

    // List projects
    let result = user.projects_list();
    assert!(result.is_ok(), "projects list failed: {:?}", result.err());

    let projects = result.unwrap();
    assert!(
        !projects.is_empty(),
        "Expected at least one project for new user"
    );
}

/// Test creating a new project.
#[tokio::test]
async fn test_create_project() {
    let harness = TestHarness::new("create_project")
        .await
        .expect("Failed to create test harness");

    let user = harness.create_user("bob");

    // Sign up first
    let signup_result = user.signup(None, TEST_PASSWORD);
    assert!(
        signup_result.is_ok(),
        "Signup failed: {:?}",
        signup_result.err()
    );

    // Create a project
    let project_name = unique_project_name();
    let result = user.projects_create(&project_name, Some("Test project created by e2e tests"));
    assert!(result.is_ok(), "projects create failed: {:?}", result.err());

    let project = result.unwrap();
    assert_eq!(project.name, project_name);
    assert!(!project.slug.is_empty(), "Project slug should not be empty");
}

/// Test project has default environments.
#[tokio::test]
async fn test_project_has_default_environments() {
    let harness = TestHarness::new("project_default_envs")
        .await
        .expect("Failed to create test harness");

    let user = harness.create_user("charlie");

    // Sign up first
    let signup_result = user.signup(None, TEST_PASSWORD);
    assert!(
        signup_result.is_ok(),
        "Signup failed: {:?}",
        signup_result.err()
    );

    // Get the first project
    let projects = user.projects_list().expect("projects list failed");
    assert!(!projects.is_empty(), "No projects found");

    // Select the project
    let use_result = user.projects_use(&projects[0].id);
    assert!(
        use_result.is_ok(),
        "projects use failed: {:?}",
        use_result.err()
    );

    // List environments
    let result = user.envs_list();
    assert!(result.is_ok(), "envs list failed: {:?}", result.err());

    let environments = result.unwrap();
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

/// Test creating multiple projects.
#[tokio::test]
async fn test_create_multiple_projects() {
    let harness = TestHarness::new("create_multiple_projects")
        .await
        .expect("Failed to create test harness");

    let user = harness.create_user("dave");

    // Sign up first
    let signup_result = user.signup(None, TEST_PASSWORD);
    assert!(
        signup_result.is_ok(),
        "Signup failed: {:?}",
        signup_result.err()
    );

    // Get initial project count
    let initial_projects = user.projects_list().expect("projects list failed");
    let initial_count = initial_projects.len();

    // Create two new projects
    for i in 0..2 {
        let project_name = format!("Multi Project {} {}", unique_project_name(), i);
        let result = user.projects_create(&project_name, None);
        assert!(result.is_ok(), "projects create failed: {:?}", result.err());
    }

    // Verify project count increased
    let final_projects = user.projects_list().expect("projects list failed");
    assert_eq!(
        final_projects.len(),
        initial_count + 2,
        "Expected 2 new projects"
    );
}

/// Test selecting a project with `projects use`.
#[tokio::test]
async fn test_projects_use() {
    let harness = TestHarness::new("projects_use")
        .await
        .expect("Failed to create test harness");

    let user = harness.create_user("eve");

    // Sign up first
    let signup_result = user.signup(None, TEST_PASSWORD);
    assert!(
        signup_result.is_ok(),
        "Signup failed: {:?}",
        signup_result.err()
    );

    // Create a project
    let project_name = unique_project_name();
    let create_result = user.projects_create(&project_name, None);
    assert!(
        create_result.is_ok(),
        "projects create failed: {:?}",
        create_result.err()
    );

    let project = create_result.unwrap();

    // Select the project by ID
    let use_result = user.projects_use(&project.id);
    assert!(
        use_result.is_ok(),
        "projects use failed: {:?}",
        use_result.err()
    );

    // Verify we can now run commands that require a project
    // (like listing flags - even if empty, it should work)
    let flags_result = user.flags_list();
    assert!(
        flags_result.is_ok(),
        "flags list should work after selecting project"
    );
}

/// Test projects are isolated between users.
#[tokio::test]
async fn test_projects_isolated_between_users() {
    let harness = TestHarness::new("projects_isolated")
        .await
        .expect("Failed to create test harness");

    // First user creates a project
    let user1 = harness.create_user("user1");
    user1.signup(None, TEST_PASSWORD).expect("Signup failed");

    let project_name = unique_project_name();
    user1
        .projects_create(&project_name, None)
        .expect("Project create failed");

    let user1_projects = user1.projects_list().expect("Projects list failed");

    // Second user signs up - shouldn't see user1's project
    let user2 = harness.create_user("user2");
    user2.signup(None, TEST_PASSWORD).expect("Signup failed");

    let user2_projects = user2.projects_list().expect("Projects list failed");

    // User2 should not see user1's custom project
    let user2_has_user1_project = user2_projects.iter().any(|p| p.name == project_name);

    assert!(
        !user2_has_user1_project,
        "User2 should not see user1's project"
    );

    // Users should have different project counts (assuming each gets a default)
    // or at least user2 should not have the explicitly named project
    assert!(
        user1_projects.len() >= 2 || user2_projects.iter().all(|p| p.name != project_name),
        "Projects should be isolated between users"
    );
}
