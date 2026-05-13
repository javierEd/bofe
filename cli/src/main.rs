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
    Applications {
        #[command(subcommand)]
        command: ApplicationsCommand,
    },
    GraphqlSchema,
}

#[derive(Subcommand)]
enum ApplicationsCommand {
    Create {
        #[arg(short, long)]
        name: String,
        #[arg(short, long)]
        expires_at: Option<NaiveDate>,
    },
    List,
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
        CliCommand::Applications {
            command: ApplicationsCommand::List,
        } => {
            let result = commands::get_all_applications().await;

            match result {
                Ok(applications) => {
                    let applications_len = applications.len();

                    println!(
                        "{} application{} found.",
                        applications_len,
                        if applications_len != 1 { "s" } else { "" }
                    );

                    for application in applications {
                        print_application(&application);
                    }
                }
                Err(err) => println!("Failed to get applications.\n\n{err}"),
            }
        }
        CliCommand::Applications {
            command: ApplicationsCommand::Create { name, expires_at },
        } => {
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
        CliCommand::GraphqlSchema => {
            let graphql_schema = GraphqlSchema::builder().finish();

            println!("{}", graphql_schema.sdl());
        }
    }
}
