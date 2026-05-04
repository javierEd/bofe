use clap::{Parser, Subcommand};

use boards_core::graphql::{GraphqlSchema, GraphqlSchemaExt};

#[derive(Parser)]
#[command(version)]
#[command(propagate_version = true)]
struct Cli {
    #[command(subcommand)]
    command: CliCommand,
}

#[derive(Subcommand)]
enum CliCommand {
    GraphqlSchema,
}

#[tokio::main]
async fn main() {
    let cli = Cli::parse();

    match &cli.command {
        CliCommand::GraphqlSchema => {
            let graphql_schema = GraphqlSchema::builder().finish();

            println!("{}", graphql_schema.sdl());
        }
    }
}
