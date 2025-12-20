use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::{Path, PathBuf};

const DEFAULT_API_URL: &str = "https://driftwatch.dev";

#[derive(Debug, Serialize, Deserialize)]
pub struct Config {
    pub token: String,
    #[serde(default = "default_api_url")]
    pub api_url: String,
}

fn default_api_url() -> String {
    DEFAULT_API_URL.to_string()
}

impl Config {
    pub fn load() -> Result<Self> {
        let api_url =
            std::env::var("DRIFTWATCH_API_URL").unwrap_or_else(|_| DEFAULT_API_URL.to_string());

        if let Ok(token) = std::env::var("DRIFTWATCH_TOKEN") {
            return Ok(Config { token, api_url });
        }

        let config_path = get_config_path()?;
        let config_str = fs::read_to_string(&config_path)
            .context("Not authenticated. Run 'driftwatch auth login' first.")?;
        let mut config: Config = toml::from_str(&config_str).context("Invalid config file")?;

        // Environment variable overrides config file
        if std::env::var("DRIFTWATCH_API_URL").is_ok() {
            config.api_url = api_url;
        }

        Ok(config)
    }
}

fn get_config_path() -> Result<PathBuf> {
    let config_dir = dirs::config_dir()
        .context("Could not determine config directory")?
        .join("driftwatch");
    Ok(config_dir.join("config.toml"))
}

pub struct ApiClient {
    client: reqwest::Client,
    base_url: String,
    token: String,
}

impl ApiClient {
    pub fn new(base_url: &str, token: &str) -> Self {
        Self {
            client: reqwest::Client::new(),
            base_url: base_url.trim_end_matches('/').to_string(),
            token: token.to_string(),
        }
    }

    async fn graphql<T: for<'de> Deserialize<'de>>(
        &self,
        query: &str,
        variables: serde_json::Value,
    ) -> Result<T> {
        let response = self
            .client
            .post(format!("{}/graphql", self.base_url))
            .header("Authorization", format!("Bearer {}", self.token))
            .header("Content-Type", "application/json")
            .json(&serde_json::json!({
                "query": query,
                "variables": variables
            }))
            .send()
            .await
            .context("Failed to send request")?;

        let status = response.status();
        let body: GraphQLResponse<T> = response.json().await.context("Failed to parse response")?;

        if let Some(errors) = body.errors {
            if !errors.is_empty() {
                return Err(anyhow::anyhow!("GraphQL error: {}", errors[0].message));
            }
        }

        body.data
            .ok_or_else(|| anyhow::anyhow!("No data in response (status: {})", status))
    }

    pub async fn list_projects(&self) -> Result<Vec<Project>> {
        let query = r#"
            query {
                projects {
                    id
                    slug
                    name
                    description
                    public
                }
            }
        "#;

        #[derive(Deserialize)]
        struct Response {
            projects: Vec<Project>,
        }

        let response: Response = self.graphql(query, serde_json::json!({})).await?;
        Ok(response.projects)
    }

    pub async fn get_project(&self, slug: &str) -> Result<Option<ProjectDetails>> {
        let query = r#"
            query GetProject($slug: String!) {
                project(slug: $slug) {
                    id
                    slug
                    name
                    description
                    public
                    branches { id name }
                    testbeds { id name }
                    benchmarks { id name }
                    measures { id name units }
                }
            }
        "#;

        #[derive(Deserialize)]
        struct Response {
            project: Option<ProjectDetails>,
        }

        let response: Response = self
            .graphql(query, serde_json::json!({ "slug": slug }))
            .await?;
        Ok(response.project)
    }

    pub async fn create_project(
        &self,
        slug: &str,
        name: &str,
        description: Option<&str>,
        public: bool,
    ) -> Result<Project> {
        let query = r#"
            mutation CreateProject($input: CreateProjectInput!) {
                createProject(input: $input) {
                    id
                    slug
                    name
                    description
                    public
                }
            }
        "#;

        #[derive(Deserialize)]
        struct Response {
            #[serde(rename = "createProject")]
            create_project: Project,
        }

        let response: Response = self
            .graphql(
                query,
                serde_json::json!({
                    "input": {
                        "slug": slug,
                        "name": name,
                        "description": description,
                        "public": public
                    }
                }),
            )
            .await?;
        Ok(response.create_project)
    }

    pub async fn create_report(
        &self,
        project_slug: &str,
        branch: &str,
        testbed: &str,
        git_hash: Option<&str>,
        pr_number: Option<i32>,
        metrics: Vec<MetricInput>,
    ) -> Result<Report> {
        let query = r#"
            mutation CreateReport($input: CreateReportInput!) {
                createReport(input: $input) {
                    id
                    gitHash
                    alerts {
                        id
                        baselineValue
                        percentChange
                    }
                }
            }
        "#;

        #[derive(Deserialize)]
        struct Response {
            #[serde(rename = "createReport")]
            create_report: Report,
        }

        let response: Response = self
            .graphql(
                query,
                serde_json::json!({
                    "input": {
                        "projectSlug": project_slug,
                        "branch": branch,
                        "testbed": testbed,
                        "gitHash": git_hash,
                        "prNumber": pr_number,
                        "metrics": metrics
                    }
                }),
            )
            .await?;
        Ok(response.create_report)
    }

    pub async fn get_flamegraph_upload_url(
        &self,
        project_slug: &str,
        file_name: &str,
    ) -> Result<FlamegraphUploadUrl> {
        let query = r#"
            mutation CreateFlamegraphUploadUrl($projectSlug: String!, $fileName: String!) {
                createFlamegraphUploadUrl(projectSlug: $projectSlug, fileName: $fileName) {
                    signedUrl
                    token
                    storagePath
                }
            }
        "#;

        #[derive(Deserialize)]
        struct Response {
            #[serde(rename = "createFlamegraphUploadUrl")]
            create_flamegraph_upload_url: FlamegraphUploadUrl,
        }

        let response: Response = self
            .graphql(
                query,
                serde_json::json!({
                    "projectSlug": project_slug,
                    "fileName": file_name
                }),
            )
            .await?;
        Ok(response.create_flamegraph_upload_url)
    }

    pub async fn upload_flamegraph_file(&self, signed_url: &str, file_path: &Path) -> Result<()> {
        let file_content = fs::read(file_path).context("Failed to read flamegraph file")?;

        let response = self
            .client
            .put(signed_url)
            .header("Content-Type", "image/svg+xml")
            .body(file_content)
            .send()
            .await
            .context("Failed to upload flamegraph")?;

        if !response.status().is_success() {
            let status = response.status();
            let body = response.text().await.unwrap_or_default();
            return Err(anyhow::anyhow!(
                "Failed to upload flamegraph: {} - {}",
                status,
                body
            ));
        }

        Ok(())
    }

    pub async fn confirm_flamegraph_upload(
        &self,
        report_id: &str,
        storage_path: &str,
        file_name: &str,
        file_size: i64,
        benchmark_name: Option<&str>,
    ) -> Result<Flamegraph> {
        let query = r#"
            mutation ConfirmFlamegraphUpload(
                $reportId: ID!,
                $storagePath: String!,
                $fileName: String!,
                $fileSize: Int!,
                $benchmarkName: String
            ) {
                confirmFlamegraphUpload(
                    reportId: $reportId,
                    storagePath: $storagePath,
                    fileName: $fileName,
                    fileSize: $fileSize,
                    benchmarkName: $benchmarkName
                ) {
                    id
                    storagePath
                    fileName
                    fileSize
                }
            }
        "#;

        #[derive(Deserialize)]
        struct Response {
            #[serde(rename = "confirmFlamegraphUpload")]
            confirm_flamegraph_upload: Flamegraph,
        }

        let response: Response = self
            .graphql(
                query,
                serde_json::json!({
                    "reportId": report_id,
                    "storagePath": storage_path,
                    "fileName": file_name,
                    "fileSize": file_size,
                    "benchmarkName": benchmark_name
                }),
            )
            .await?;
        Ok(response.confirm_flamegraph_upload)
    }
}

#[derive(Debug, Deserialize)]
struct GraphQLResponse<T> {
    data: Option<T>,
    errors: Option<Vec<GraphQLError>>,
}

#[derive(Debug, Deserialize)]
struct GraphQLError {
    message: String,
}

#[allow(dead_code)]
#[derive(Debug, Deserialize)]
pub struct Project {
    pub id: String,
    pub slug: String,
    pub name: String,
    pub description: Option<String>,
    pub public: bool,
}

#[allow(dead_code)]
#[derive(Debug, Deserialize)]
pub struct ProjectDetails {
    pub id: String,
    pub slug: String,
    pub name: String,
    pub description: Option<String>,
    pub public: bool,
    pub branches: Vec<Branch>,
    pub testbeds: Vec<Testbed>,
    pub benchmarks: Vec<Benchmark>,
    pub measures: Vec<Measure>,
}

#[allow(dead_code)]
#[derive(Debug, Deserialize)]
pub struct Branch {
    pub id: String,
    pub name: String,
}

#[allow(dead_code)]
#[derive(Debug, Deserialize)]
pub struct Testbed {
    pub id: String,
    pub name: String,
}

#[allow(dead_code)]
#[derive(Debug, Deserialize)]
pub struct Benchmark {
    pub id: String,
    pub name: String,
}

#[allow(dead_code)]
#[derive(Debug, Deserialize)]
pub struct Measure {
    pub id: String,
    pub name: String,
    pub units: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct MetricInput {
    pub benchmark: String,
    pub measure: String,
    pub value: f64,
    #[serde(rename = "lowerValue")]
    pub lower_value: Option<f64>,
    #[serde(rename = "upperValue")]
    pub upper_value: Option<f64>,
}

#[allow(dead_code)]
#[derive(Debug, Deserialize)]
pub struct Report {
    pub id: String,
    #[serde(rename = "gitHash")]
    pub git_hash: Option<String>,
    pub alerts: Vec<Alert>,
}

#[allow(dead_code)]
#[derive(Debug, Deserialize)]
pub struct Alert {
    pub id: String,
    #[serde(rename = "baselineValue")]
    pub baseline_value: f64,
    #[serde(rename = "percentChange")]
    pub percent_change: f64,
}

#[allow(dead_code)]
#[derive(Debug, Deserialize)]
pub struct FlamegraphUploadUrl {
    #[serde(rename = "signedUrl")]
    pub signed_url: String,
    pub token: String,
    #[serde(rename = "storagePath")]
    pub storage_path: String,
}

#[allow(dead_code)]
#[derive(Debug, Deserialize)]
pub struct Flamegraph {
    pub id: String,
    #[serde(rename = "storagePath")]
    pub storage_path: String,
    #[serde(rename = "fileName")]
    pub file_name: String,
    #[serde(rename = "fileSize")]
    pub file_size: i64,
}
