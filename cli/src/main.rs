use chrono::NaiveDate;
use clap::{Parser, Subcommand};

use boards_core::commands;
use boards_core::graphql::{GraphqlSchema, GraphqlSchemaExt};
use boards_core::models::Application;
use boards_core::params::ApplicationParams;

#[derive(Parser)]
#[command(version)]
#[command(propagate_version = true)]
struct Cli {
    #[command(subcommand)]
    command: CliCommand,
}

#[derive(Subcommand)]
enum CliCommand {
    CreateApplication {
        #[arg(short, long)]
        name: String,
        #[arg(short, long)]
        expires_at: Option<NaiveDate>,
    },
    GraphqlSchema,
}

fn print_application(application: &Application) {
    println!(
        "ID: {}\nName: {}\nToken: {}\nExpires At: {}\nCreated at: {}\nUpdated at: {}",
        application.id,
        application.name,
        application.token,
        application.expires_at,
        application.created_at,
        application
            .updated_at
            .map(|updated_at| updated_at.to_string())
            .unwrap_or_else(|| "None".to_owned())
    );
}

#[tokio::main]
async fn main() {
    let cli = Cli::parse();

    match &cli.command {
        CliCommand::GraphqlSchema => {
            let graphql_schema = GraphqlSchema::builder().finish();

            println!("{}", graphql_schema.sdl());
        }
        CliCommand::CreateApplication { name, expires_at } => {
            let result = commands::insert_application(ApplicationParams {
                name: name.clone(),
                expires_at: *expires_at,
            })
            .await;

            match result {
                Ok(application) => {
                    println!("Application created successfully.");
                    print_application(&application);
                }
                Err(err) => println!("Failed to create application.\n\n{err}"),
            }
        }
    }
}
