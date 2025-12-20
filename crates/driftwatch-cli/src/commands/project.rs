use anyhow::Result;
use clap::Subcommand;

use crate::api::{ApiClient, Config};

#[derive(Subcommand)]
pub enum ProjectCommands {
    List,
    Create {
        #[arg(long)]
        slug: String,
        #[arg(long)]
        name: String,
        #[arg(long)]
        description: Option<String>,
        #[arg(long, default_value = "false")]
        public: bool,
    },
    Show {
        slug: String,
    },
}

pub async fn handle(command: ProjectCommands, api_url: &str) -> Result<()> {
    let config = Config::load()?;
    let client = ApiClient::new(api_url, &config.token);

    match command {
        ProjectCommands::List => list(&client).await,
        ProjectCommands::Create {
            slug,
            name,
            description,
            public,
        } => create(&client, &slug, &name, description.as_deref(), public).await,
        ProjectCommands::Show { slug } => show(&client, &slug).await,
    }
}

async fn list(client: &ApiClient) -> Result<()> {
    let projects = client.list_projects().await?;

    if projects.is_empty() {
        println!("No projects found.");
        println!(
            "Create one with: benchctl project create --slug my-project --name \"My Project\""
        );
        return Ok(());
    }

    println!("{:<20} {:<30} PUBLIC", "SLUG", "NAME");
    println!("{}", "-".repeat(60));

    for project in projects {
        let public = if project.public { "yes" } else { "no" };
        println!("{:<20} {:<30} {}", project.slug, project.name, public);
    }

    Ok(())
}

async fn create(
    client: &ApiClient,
    slug: &str,
    name: &str,
    description: Option<&str>,
    public: bool,
) -> Result<()> {
    let project = client
        .create_project(slug, name, description, public)
        .await?;
    println!("Created project: {} ({})", project.name, project.slug);
    Ok(())
}

async fn show(client: &ApiClient, slug: &str) -> Result<()> {
    let project = client.get_project(slug).await?;

    match project {
        Some(p) => {
            println!("Name: {}", p.name);
            println!("Slug: {}", p.slug);
            println!("Public: {}", p.public);
            if let Some(desc) = p.description {
                println!("Description: {}", desc);
            }

            println!("\nBranches: {}", p.branches.len());
            for branch in &p.branches {
                println!("  - {}", branch.name);
            }

            println!("\nTestbeds: {}", p.testbeds.len());
            for testbed in &p.testbeds {
                println!("  - {}", testbed.name);
            }

            println!("\nBenchmarks: {}", p.benchmarks.len());
            for benchmark in &p.benchmarks {
                println!("  - {}", benchmark.name);
            }
        }
        None => {
            println!("Project not found: {}", slug);
        }
    }

    Ok(())
}
