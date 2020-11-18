use color_eyre::{self, Help};
use eyre::*;
use graphql_client::*;
use tokio;

type URI = String;

#[derive(GraphQLQuery)]
#[graphql(
    schema_path = "src/schema.graphql",
    query_path = "src/query.graphql",
    response_derives = "Debug"
)]
struct RepoView;

#[tokio::main]
async fn main() -> Result<(), eyre::Error> {
    env_logger::init();
    color_eyre::install()?;

    let github_api_token = std::env::var("GITHUB_API_TOKEN").suggestion("Set GITHUB_API_TOKEN")?;

    let (owner, name) = ("graphql-rust", "graphql-client");

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
    println!("{:?}", response_body);

    Ok(())
}
