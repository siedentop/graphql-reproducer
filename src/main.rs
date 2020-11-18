use color_eyre::{self, Help};
use eyre::*;
use graphql_client::*;
use log::*;
use tokio;

type URI = String;

#[derive(GraphQLQuery)]
#[graphql(
    schema_path = "src/schema.graphql",
    query_path = "src/query.graphql",
    response_derives = "Debug"
)]
struct RepoView;

fn parse_repo_name(repo_name: &str) -> Result<(&str, &str), eyre::Error> {
    let mut parts = repo_name.split('/');
    match (parts.next(), parts.next()) {
        (Some(owner), Some(name)) => Ok((owner, name)),
        _ => Err(format_err!("wrong format for the repository name param (we expect something like facebook/graphql)"))
    }
}

#[tokio::main]
async fn main() -> Result<(), eyre::Error> {
    env_logger::init();
    color_eyre::install()?;

    let github_api_token = std::env::var("GITHUB_API_TOKEN").suggestion("Set GITHUB_API_TOKEN")?;

    let repo = "graphql-rust/graphql-client";
    let (owner, name) = parse_repo_name(&repo).unwrap_or(("tomhoule", "graphql-client"));

    let q = RepoView::build_query(repo_view::Variables {
        owner: owner.to_string(),
        name: name.to_string(),
    });

    let client = reqwest::Client::new();

    let res = client
        .post("https://api.github.com/graphql")
        .bearer_auth(github_api_token)
        .json(&q)
        .send()
        .await?;

    let context = format!("Response: {:?}", res);
    let response_body: Response<repo_view::ResponseData> = res.json().await.context(context)?;
    info!("{:?}", response_body);

    if let Some(errors) = response_body.errors {
        println!("there are errors:");

        for error in &errors {
            println!("{:?}", error);
        }
    }

    let response_data: repo_view::ResponseData = response_body.data.expect("missing response data");

    let stars: Option<i64> = response_data
        .repository
        .as_ref()
        .map(|repo| repo.stargazers.total_count);

    println!("{}/{} - 🌟 {}", owner, name, stars.unwrap_or(0),);

    for issue in &response_data
        .repository
        .expect("missing repository")
        .issues
        .nodes
        .expect("issue nodes is null")
    {
        if let Some(issue) = issue {
            println!("{}, {}", issue.title, issue.comments.total_count);
        }
    }

    Ok(())
}
