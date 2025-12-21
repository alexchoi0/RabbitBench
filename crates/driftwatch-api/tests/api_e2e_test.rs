mod common;

use serde::Deserialize;

#[derive(Debug, Deserialize)]
struct ProjectsData {
    projects: Vec<ProjectData>,
}

#[derive(Debug, Deserialize)]
struct ProjectData {
    id: String,
    slug: String,
    name: String,
    description: Option<String>,
    public: bool,
    #[serde(rename = "githubRepo")]
    github_repo: Option<String>,
    #[serde(rename = "githubPrComments")]
    github_pr_comments: bool,
    #[serde(rename = "githubStatusChecks")]
    github_status_checks: bool,
    #[serde(rename = "hasGithubToken")]
    has_github_token: bool,
}

#[derive(Debug, Deserialize)]
struct SingleProjectData {
    project: Option<ProjectData>,
}

#[derive(Debug, Deserialize)]
struct CreateProjectData {
    #[serde(rename = "createProject")]
    create_project: ProjectData,
}

#[derive(Debug, Deserialize)]
struct UpdateProjectData {
    #[serde(rename = "updateProject")]
    update_project: ProjectData,
}

#[derive(Debug, Deserialize)]
struct DeleteProjectData {
    #[serde(rename = "deleteProject")]
    delete_project: bool,
}

#[derive(Debug, Deserialize)]
struct ApiTokensData {
    #[serde(rename = "apiTokens")]
    api_tokens: Vec<ApiTokenData>,
}

#[derive(Debug, Deserialize)]
struct ApiTokenData {
    id: String,
    name: String,
    #[serde(rename = "createdAt")]
    created_at: String,
}

#[derive(Debug, Deserialize)]
struct CreateApiTokenData {
    #[serde(rename = "createApiToken")]
    create_api_token: CreateApiTokenResultData,
}

#[derive(Debug, Deserialize)]
struct CreateApiTokenResultData {
    token: ApiTokenData,
    secret: String,
}

#[derive(Debug, Deserialize)]
struct RevokeApiTokenData {
    #[serde(rename = "revokeApiToken")]
    revoke_api_token: bool,
}

#[derive(Debug, Deserialize)]
struct UpdateGithubSettingsData {
    #[serde(rename = "updateGithubSettings")]
    update_github_settings: ProjectData,
}

#[derive(Debug, Deserialize)]
struct ThresholdData {
    id: String,
    #[serde(rename = "upperBoundary")]
    upper_boundary: Option<f64>,
    #[serde(rename = "lowerBoundary")]
    lower_boundary: Option<f64>,
    #[serde(rename = "minSampleSize")]
    min_sample_size: i32,
}

#[derive(Debug, Deserialize)]
struct CreateThresholdData {
    #[serde(rename = "createThreshold")]
    create_threshold: ThresholdData,
}

#[derive(Debug, Deserialize)]
struct DeleteThresholdData {
    #[serde(rename = "deleteThreshold")]
    delete_threshold: bool,
}

#[derive(Debug, Deserialize)]
struct ProjectWithMeasuresData {
    project: Option<ProjectWithMeasures>,
}

#[derive(Debug, Deserialize)]
struct ProjectWithMeasures {
    id: String,
    measures: Vec<MeasureData>,
}

#[derive(Debug, Deserialize)]
struct MeasureData {
    id: String,
    name: String,
}

const CREATE_PROJECT: &str = r#"
mutation CreateProject($input: CreateProjectInput!) {
    createProject(input: $input) {
        id
        slug
        name
        description
        public
        githubRepo
        githubPrComments
        githubStatusChecks
        hasGithubToken
    }
}
"#;

const GET_PROJECTS: &str = r#"
query GetProjects {
    projects {
        id
        slug
        name
        description
        public
        githubRepo
        githubPrComments
        githubStatusChecks
        hasGithubToken
    }
}
"#;

const GET_PROJECT: &str = r#"
query GetProject($slug: String!) {
    project(slug: $slug) {
        id
        slug
        name
        description
        public
        githubRepo
        githubPrComments
        githubStatusChecks
        hasGithubToken
    }
}
"#;

const GET_PROJECT_WITH_MEASURES: &str = r#"
query GetProjectWithMeasures($slug: String!) {
    project(slug: $slug) {
        id
        measures {
            id
            name
        }
    }
}
"#;

const UPDATE_PROJECT: &str = r#"
mutation UpdateProject($slug: String!, $input: UpdateProjectInput!) {
    updateProject(slug: $slug, input: $input) {
        id
        slug
        name
        description
        public
        githubRepo
        githubPrComments
        githubStatusChecks
        hasGithubToken
    }
}
"#;

const DELETE_PROJECT: &str = r#"
mutation DeleteProject($slug: String!) {
    deleteProject(slug: $slug)
}
"#;

const UPDATE_GITHUB_SETTINGS: &str = r#"
mutation UpdateGithubSettings($slug: String!, $input: GitHubSettingsInput!) {
    updateGithubSettings(slug: $slug, input: $input) {
        id
        slug
        name
        description
        public
        githubRepo
        githubPrComments
        githubStatusChecks
        hasGithubToken
    }
}
"#;

const GET_API_TOKENS: &str = r#"
query GetApiTokens {
    apiTokens {
        id
        name
        createdAt
    }
}
"#;

const CREATE_API_TOKEN: &str = r#"
mutation CreateApiToken($name: String!) {
    createApiToken(name: $name) {
        token {
            id
            name
            createdAt
        }
        secret
    }
}
"#;

const REVOKE_API_TOKEN: &str = r#"
mutation RevokeApiToken($id: ID!) {
    revokeApiToken(id: $id)
}
"#;

const CREATE_THRESHOLD: &str = r#"
mutation CreateThreshold($input: CreateThresholdInput!) {
    createThreshold(input: $input) {
        id
        upperBoundary
        lowerBoundary
        minSampleSize
    }
}
"#;

const DELETE_THRESHOLD: &str = r#"
mutation DeleteThreshold($id: ID!) {
    deleteThreshold(id: $id)
}
"#;

#[tokio::test]
async fn test_create_and_get_project() {
    let server = test_server!();
    let token = server.create_test_token("user-1");

    let result: CreateProjectData = server
        .graphql(
            CREATE_PROJECT,
            Some(serde_json::json!({
                "input": {
                    "slug": "my-project",
                    "name": "My Project",
                    "description": "A test project"
                }
            })),
            Some(&token),
        )
        .await
        .unwrap();

    assert_eq!(result.create_project.slug, "my-project");
    assert_eq!(result.create_project.name, "My Project");
    assert_eq!(
        result.create_project.description,
        Some("A test project".to_string())
    );
    assert!(!result.create_project.public);

    let result: SingleProjectData = server
        .graphql(
            GET_PROJECT,
            Some(serde_json::json!({ "slug": "my-project" })),
            Some(&token),
        )
        .await
        .unwrap();

    let project = result.project.unwrap();
    assert_eq!(project.slug, "my-project");
    assert_eq!(project.name, "My Project");
}

#[tokio::test]
async fn test_list_projects() {
    let server = test_server!();
    let token = server.create_test_token("user-1");

    for i in 0..3 {
        let _: CreateProjectData = server
            .graphql(
                CREATE_PROJECT,
                Some(serde_json::json!({
                    "input": {
                        "slug": format!("project-{}", i),
                        "name": format!("Project {}", i)
                    }
                })),
                Some(&token),
            )
            .await
            .unwrap();
    }

    let result: ProjectsData = server
        .graphql(GET_PROJECTS, None, Some(&token))
        .await
        .unwrap();

    assert_eq!(result.projects.len(), 3);
}

#[tokio::test]
async fn test_update_project() {
    let server = test_server!();
    let token = server.create_test_token("user-1");

    let _: CreateProjectData = server
        .graphql(
            CREATE_PROJECT,
            Some(serde_json::json!({
                "input": {
                    "slug": "update-test",
                    "name": "Original Name"
                }
            })),
            Some(&token),
        )
        .await
        .unwrap();

    let result: UpdateProjectData = server
        .graphql(
            UPDATE_PROJECT,
            Some(serde_json::json!({
                "slug": "update-test",
                "input": {
                    "name": "Updated Name",
                    "description": "New description",
                    "public": true
                }
            })),
            Some(&token),
        )
        .await
        .unwrap();

    assert_eq!(result.update_project.name, "Updated Name");
    assert_eq!(
        result.update_project.description,
        Some("New description".to_string())
    );
    assert!(result.update_project.public);
}

#[tokio::test]
async fn test_delete_project() {
    let server = test_server!();
    let token = server.create_test_token("user-1");

    let _: CreateProjectData = server
        .graphql(
            CREATE_PROJECT,
            Some(serde_json::json!({
                "input": {
                    "slug": "delete-me",
                    "name": "To Be Deleted"
                }
            })),
            Some(&token),
        )
        .await
        .unwrap();

    let result: DeleteProjectData = server
        .graphql(
            DELETE_PROJECT,
            Some(serde_json::json!({ "slug": "delete-me" })),
            Some(&token),
        )
        .await
        .unwrap();

    assert!(result.delete_project);

    let result: SingleProjectData = server
        .graphql(
            GET_PROJECT,
            Some(serde_json::json!({ "slug": "delete-me" })),
            Some(&token),
        )
        .await
        .unwrap();

    assert!(result.project.is_none());
}

#[tokio::test]
async fn test_duplicate_slug_rejected() {
    let server = test_server!();
    let token = server.create_test_token("user-1");

    let _: CreateProjectData = server
        .graphql(
            CREATE_PROJECT,
            Some(serde_json::json!({
                "input": {
                    "slug": "unique-slug",
                    "name": "First Project"
                }
            })),
            Some(&token),
        )
        .await
        .unwrap();

    let result = server
        .graphql::<CreateProjectData>(
            CREATE_PROJECT,
            Some(serde_json::json!({
                "input": {
                    "slug": "unique-slug",
                    "name": "Duplicate Project"
                }
            })),
            Some(&token),
        )
        .await;

    assert!(result.errors.is_some());
}

#[tokio::test]
async fn test_user_isolation() {
    let server = test_server!();
    let token_user1 = server.create_test_token("user-1");
    let token_user2 = server.create_test_token("user-2");

    let _: CreateProjectData = server
        .graphql(
            CREATE_PROJECT,
            Some(serde_json::json!({
                "input": {
                    "slug": "user1-project",
                    "name": "User 1 Project"
                }
            })),
            Some(&token_user1),
        )
        .await
        .unwrap();

    let _: CreateProjectData = server
        .graphql(
            CREATE_PROJECT,
            Some(serde_json::json!({
                "input": {
                    "slug": "user2-project",
                    "name": "User 2 Project"
                }
            })),
            Some(&token_user2),
        )
        .await
        .unwrap();

    let user1_projects: ProjectsData = server
        .graphql(GET_PROJECTS, None, Some(&token_user1))
        .await
        .unwrap();
    assert_eq!(user1_projects.projects.len(), 1);
    assert_eq!(user1_projects.projects[0].slug, "user1-project");

    let user2_projects: ProjectsData = server
        .graphql(GET_PROJECTS, None, Some(&token_user2))
        .await
        .unwrap();
    assert_eq!(user2_projects.projects.len(), 1);
    assert_eq!(user2_projects.projects[0].slug, "user2-project");

    let result: SingleProjectData = server
        .graphql(
            GET_PROJECT,
            Some(serde_json::json!({ "slug": "user1-project" })),
            Some(&token_user2),
        )
        .await
        .unwrap();
    assert!(result.project.is_none());
}

#[tokio::test]
async fn test_github_settings() {
    let server = test_server!();
    let token = server.create_test_token("user-1");

    let _: CreateProjectData = server
        .graphql(
            CREATE_PROJECT,
            Some(serde_json::json!({
                "input": {
                    "slug": "github-test",
                    "name": "GitHub Test"
                }
            })),
            Some(&token),
        )
        .await
        .unwrap();

    let result: UpdateGithubSettingsData = server
        .graphql(
            UPDATE_GITHUB_SETTINGS,
            Some(serde_json::json!({
                "slug": "github-test",
                "input": {
                    "githubRepo": "owner/repo",
                    "githubToken": "ghp_test_token_123",
                    "githubPrComments": true,
                    "githubStatusChecks": true
                }
            })),
            Some(&token),
        )
        .await
        .unwrap();

    assert_eq!(
        result.update_github_settings.github_repo,
        Some("owner/repo".to_string())
    );
    assert!(result.update_github_settings.github_pr_comments);
    assert!(result.update_github_settings.github_status_checks);
    assert!(result.update_github_settings.has_github_token);
}

#[tokio::test]
async fn test_api_token_lifecycle() {
    let server = test_server!();
    let token = server.create_test_token("user-1");

    let tokens: ApiTokensData = server
        .graphql(GET_API_TOKENS, None, Some(&token))
        .await
        .unwrap();
    assert!(tokens.api_tokens.is_empty());

    let result: CreateApiTokenData = server
        .graphql(
            CREATE_API_TOKEN,
            Some(serde_json::json!({ "name": "CI Token" })),
            Some(&token),
        )
        .await
        .unwrap();

    assert_eq!(result.create_api_token.token.name, "CI Token");
    assert!(result.create_api_token.secret.starts_with("dw_"));
    let token_id = result.create_api_token.token.id.clone();

    let tokens: ApiTokensData = server
        .graphql(GET_API_TOKENS, None, Some(&token))
        .await
        .unwrap();
    assert_eq!(tokens.api_tokens.len(), 1);

    let result: RevokeApiTokenData = server
        .graphql(
            REVOKE_API_TOKEN,
            Some(serde_json::json!({ "id": token_id })),
            Some(&token),
        )
        .await
        .unwrap();
    assert!(result.revoke_api_token);

    let tokens: ApiTokensData = server
        .graphql(GET_API_TOKENS, None, Some(&token))
        .await
        .unwrap();
    assert!(tokens.api_tokens.is_empty());
}

#[tokio::test]
async fn test_multiple_api_tokens() {
    let server = test_server!();
    let token = server.create_test_token("user-1");

    for name in ["Development", "CI/CD", "Production"] {
        let _: CreateApiTokenData = server
            .graphql(
                CREATE_API_TOKEN,
                Some(serde_json::json!({ "name": name })),
                Some(&token),
            )
            .await
            .unwrap();
    }

    let tokens: ApiTokensData = server
        .graphql(GET_API_TOKENS, None, Some(&token))
        .await
        .unwrap();
    assert_eq!(tokens.api_tokens.len(), 3);
}

#[tokio::test]
async fn test_threshold_creation() {
    let server = test_server!();
    let token = server.create_test_token("user-1");

    let _: CreateProjectData = server
        .graphql(
            CREATE_PROJECT,
            Some(serde_json::json!({
                "input": {
                    "slug": "threshold-test",
                    "name": "Threshold Test"
                }
            })),
            Some(&token),
        )
        .await
        .unwrap();

    let project: ProjectWithMeasuresData = server
        .graphql(
            GET_PROJECT_WITH_MEASURES,
            Some(serde_json::json!({ "slug": "threshold-test" })),
            Some(&token),
        )
        .await
        .unwrap();

    let measure_id = &project.project.unwrap().measures[0].id;

    let result: CreateThresholdData = server
        .graphql(
            CREATE_THRESHOLD,
            Some(serde_json::json!({
                "input": {
                    "projectSlug": "threshold-test",
                    "measureId": measure_id,
                    "upperBoundary": 1.5,
                    "lowerBoundary": 0.5,
                    "minSampleSize": 5
                }
            })),
            Some(&token),
        )
        .await
        .unwrap();

    assert_eq!(result.create_threshold.upper_boundary, Some(1.5));
    assert_eq!(result.create_threshold.lower_boundary, Some(0.5));
    assert_eq!(result.create_threshold.min_sample_size, 5);
}

#[tokio::test]
async fn test_threshold_deletion() {
    let server = test_server!();
    let token = server.create_test_token("user-1");

    let _: CreateProjectData = server
        .graphql(
            CREATE_PROJECT,
            Some(serde_json::json!({
                "input": {
                    "slug": "threshold-delete-test",
                    "name": "Threshold Delete Test"
                }
            })),
            Some(&token),
        )
        .await
        .unwrap();

    let project: ProjectWithMeasuresData = server
        .graphql(
            GET_PROJECT_WITH_MEASURES,
            Some(serde_json::json!({ "slug": "threshold-delete-test" })),
            Some(&token),
        )
        .await
        .unwrap();

    let measure_id = &project.project.unwrap().measures[0].id;

    let result: CreateThresholdData = server
        .graphql(
            CREATE_THRESHOLD,
            Some(serde_json::json!({
                "input": {
                    "projectSlug": "threshold-delete-test",
                    "measureId": measure_id,
                    "upperBoundary": 2.0
                }
            })),
            Some(&token),
        )
        .await
        .unwrap();

    let threshold_id = result.create_threshold.id;

    let result: DeleteThresholdData = server
        .graphql(
            DELETE_THRESHOLD,
            Some(serde_json::json!({ "id": threshold_id })),
            Some(&token),
        )
        .await
        .unwrap();

    assert!(result.delete_threshold);
}

#[tokio::test]
async fn test_cross_user_threshold_rejection() {
    let server = test_server!();
    let token_user1 = server.create_test_token("user-1");
    let token_user2 = server.create_test_token("user-2");

    let _: CreateProjectData = server
        .graphql(
            CREATE_PROJECT,
            Some(serde_json::json!({
                "input": {
                    "slug": "user1-only",
                    "name": "User 1 Only"
                }
            })),
            Some(&token_user1),
        )
        .await
        .unwrap();

    let project: ProjectWithMeasuresData = server
        .graphql(
            GET_PROJECT_WITH_MEASURES,
            Some(serde_json::json!({ "slug": "user1-only" })),
            Some(&token_user1),
        )
        .await
        .unwrap();

    let measure_id = &project.project.unwrap().measures[0].id;

    let result = server
        .graphql::<CreateThresholdData>(
            CREATE_THRESHOLD,
            Some(serde_json::json!({
                "input": {
                    "projectSlug": "user1-only",
                    "measureId": measure_id,
                    "upperBoundary": 1.5
                }
            })),
            Some(&token_user2),
        )
        .await;

    assert!(result.errors.is_some());
}

#[tokio::test]
async fn test_unauthenticated_requests_rejected() {
    let server = test_server!();

    let result = server
        .graphql::<ProjectsData>(GET_PROJECTS, None, None)
        .await;

    assert!(result.errors.is_some());
}

#[tokio::test]
async fn test_project_workflow_lifecycle() {
    let server = test_server!();
    let token = server.create_test_token("developer-1");

    let _: CreateProjectData = server
        .graphql(
            CREATE_PROJECT,
            Some(serde_json::json!({
                "input": {
                    "slug": "my-app",
                    "name": "My Application",
                    "description": "Performance benchmarking for my app"
                }
            })),
            Some(&token),
        )
        .await
        .unwrap();

    let _: CreateApiTokenData = server
        .graphql(
            CREATE_API_TOKEN,
            Some(serde_json::json!({ "name": "CI Pipeline" })),
            Some(&token),
        )
        .await
        .unwrap();

    let _: UpdateGithubSettingsData = server
        .graphql(
            UPDATE_GITHUB_SETTINGS,
            Some(serde_json::json!({
                "slug": "my-app",
                "input": {
                    "githubRepo": "developer-1/my-app",
                    "githubToken": "ghp_abc123",
                    "githubPrComments": true,
                    "githubStatusChecks": true
                }
            })),
            Some(&token),
        )
        .await
        .unwrap();

    let project: ProjectWithMeasuresData = server
        .graphql(
            GET_PROJECT_WITH_MEASURES,
            Some(serde_json::json!({ "slug": "my-app" })),
            Some(&token),
        )
        .await
        .unwrap();

    let measure_id = &project.project.unwrap().measures[0].id;

    let _: CreateThresholdData = server
        .graphql(
            CREATE_THRESHOLD,
            Some(serde_json::json!({
                "input": {
                    "projectSlug": "my-app",
                    "measureId": measure_id,
                    "upperBoundary": 1.2,
                    "minSampleSize": 10
                }
            })),
            Some(&token),
        )
        .await
        .unwrap();

    let project: SingleProjectData = server
        .graphql(
            GET_PROJECT,
            Some(serde_json::json!({ "slug": "my-app" })),
            Some(&token),
        )
        .await
        .unwrap();

    let p = project.project.unwrap();
    assert_eq!(p.slug, "my-app");
    assert_eq!(p.github_repo, Some("developer-1/my-app".to_string()));
    assert!(p.github_pr_comments);
    assert!(p.github_status_checks);
    assert!(p.has_github_token);
}

#[tokio::test]
async fn test_project_default_measure_created() {
    let server = test_server!();
    let token = server.create_test_token("user-1");

    let _: CreateProjectData = server
        .graphql(
            CREATE_PROJECT,
            Some(serde_json::json!({
                "input": {
                    "slug": "measure-test",
                    "name": "Measure Test"
                }
            })),
            Some(&token),
        )
        .await
        .unwrap();

    let project: ProjectWithMeasuresData = server
        .graphql(
            GET_PROJECT_WITH_MEASURES,
            Some(serde_json::json!({ "slug": "measure-test" })),
            Some(&token),
        )
        .await
        .unwrap();

    let measures = project.project.unwrap().measures;
    assert_eq!(measures.len(), 1);
    assert_eq!(measures[0].name, "latency");
}

#[tokio::test]
async fn test_cannot_delete_other_users_project() {
    let server = test_server!();
    let token_user1 = server.create_test_token("user-1");
    let token_user2 = server.create_test_token("user-2");

    let _: CreateProjectData = server
        .graphql(
            CREATE_PROJECT,
            Some(serde_json::json!({
                "input": {
                    "slug": "protected",
                    "name": "Protected Project"
                }
            })),
            Some(&token_user1),
        )
        .await
        .unwrap();

    let result = server
        .graphql::<DeleteProjectData>(
            DELETE_PROJECT,
            Some(serde_json::json!({ "slug": "protected" })),
            Some(&token_user2),
        )
        .await;

    assert!(result.errors.is_some());

    let project: SingleProjectData = server
        .graphql(
            GET_PROJECT,
            Some(serde_json::json!({ "slug": "protected" })),
            Some(&token_user1),
        )
        .await
        .unwrap();

    assert!(project.project.is_some());
}

#[tokio::test]
async fn test_cannot_update_other_users_project() {
    let server = test_server!();
    let token_user1 = server.create_test_token("user-1");
    let token_user2 = server.create_test_token("user-2");

    let _: CreateProjectData = server
        .graphql(
            CREATE_PROJECT,
            Some(serde_json::json!({
                "input": {
                    "slug": "private-project",
                    "name": "Private Project"
                }
            })),
            Some(&token_user1),
        )
        .await
        .unwrap();

    let result = server
        .graphql::<UpdateProjectData>(
            UPDATE_PROJECT,
            Some(serde_json::json!({
                "slug": "private-project",
                "input": {
                    "name": "Hacked!"
                }
            })),
            Some(&token_user2),
        )
        .await;

    assert!(result.errors.is_some());
}

#[tokio::test]
async fn test_cannot_revoke_other_users_token() {
    let server = test_server!();
    let token_user1 = server.create_test_token("user-1");
    let token_user2 = server.create_test_token("user-2");

    let result: CreateApiTokenData = server
        .graphql(
            CREATE_API_TOKEN,
            Some(serde_json::json!({ "name": "User 1 Token" })),
            Some(&token_user1),
        )
        .await
        .unwrap();

    let token_id = result.create_api_token.token.id;

    let result = server
        .graphql::<RevokeApiTokenData>(
            REVOKE_API_TOKEN,
            Some(serde_json::json!({ "id": token_id })),
            Some(&token_user2),
        )
        .await;

    assert!(result.errors.is_some());

    let tokens: ApiTokensData = server
        .graphql(GET_API_TOKENS, None, Some(&token_user1))
        .await
        .unwrap();
    assert_eq!(tokens.api_tokens.len(), 1);
}

#[tokio::test]
async fn test_cache_invalidation_on_project_update() {
    let server = test_server!();
    let token = server.create_test_token("user-1");

    let _: CreateProjectData = server
        .graphql(
            CREATE_PROJECT,
            Some(serde_json::json!({
                "input": {
                    "slug": "cache-test",
                    "name": "Original Name"
                }
            })),
            Some(&token),
        )
        .await
        .unwrap();

    let project: SingleProjectData = server
        .graphql(
            GET_PROJECT,
            Some(serde_json::json!({ "slug": "cache-test" })),
            Some(&token),
        )
        .await
        .unwrap();
    assert_eq!(project.project.unwrap().name, "Original Name");

    let _: UpdateProjectData = server
        .graphql(
            UPDATE_PROJECT,
            Some(serde_json::json!({
                "slug": "cache-test",
                "input": {
                    "name": "Updated Name"
                }
            })),
            Some(&token),
        )
        .await
        .unwrap();

    let project: SingleProjectData = server
        .graphql(
            GET_PROJECT,
            Some(serde_json::json!({ "slug": "cache-test" })),
            Some(&token),
        )
        .await
        .unwrap();
    assert_eq!(project.project.unwrap().name, "Updated Name");
}

#[tokio::test]
async fn test_large_project_list() {
    let server = test_server!();
    let token = server.create_test_token("user-1");

    for i in 0..20 {
        let _: CreateProjectData = server
            .graphql(
                CREATE_PROJECT,
                Some(serde_json::json!({
                    "input": {
                        "slug": format!("project-{:02}", i),
                        "name": format!("Project {}", i)
                    }
                })),
                Some(&token),
            )
            .await
            .unwrap();
    }

    let result: ProjectsData = server
        .graphql(GET_PROJECTS, None, Some(&token))
        .await
        .unwrap();

    assert_eq!(result.projects.len(), 20);
}
