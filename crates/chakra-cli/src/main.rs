//! Chakra ORM Command-Line Interface

use clap::{Parser, Subcommand};
use colored::Colorize;
use std::path::PathBuf;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

mod commands;

#[derive(Parser)]
#[command(name = "chakra")]
#[command(author = "Chakra ORM Contributors")]
#[command(version = env!("CARGO_PKG_VERSION"))]
#[command(about = "Chakra ORM - Next-generation database toolkit", long_about = None)]
struct Cli {
    /// Configuration file path
    #[arg(short, long, default_value = "chakra.toml")]
    config: PathBuf,

    /// Database URL (overrides config)
    #[arg(long, env = "DATABASE_URL")]
    database_url: Option<String>,

    /// Verbose output
    #[arg(short, long)]
    verbose: bool,

    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Initialize a new Chakra project
    Init {
        /// Project directory
        #[arg(default_value = ".")]
        path: PathBuf,
    },

    /// Database operations
    Db {
        #[command(subcommand)]
        command: DbCommands,
    },

    /// Migration commands
    Migrate {
        #[command(subcommand)]
        command: MigrateCommands,
    },

    /// Generate code from database
    Generate {
        #[command(subcommand)]
        command: GenerateCommands,
    },

    /// Schema management
    Schema {
        #[command(subcommand)]
        command: SchemaCommands,
    },
}

#[derive(Subcommand)]
enum DbCommands {
    /// Create the database
    Create,

    /// Drop the database
    Drop {
        /// Force drop without confirmation
        #[arg(short, long)]
        force: bool,
    },

    /// Reset the database (drop + create + migrate)
    Reset {
        /// Force reset without confirmation
        #[arg(short, long)]
        force: bool,
    },

    /// Show database status
    Status,

    /// Open a database shell
    Shell,
}

#[derive(Subcommand)]
enum MigrateCommands {
    /// Create a new migration
    New {
        /// Migration name
        name: String,

        /// App/module name
        #[arg(short, long)]
        app: Option<String>,
    },

    /// Apply pending migrations
    Up {
        /// Target migration ID
        #[arg(short, long)]
        target: Option<String>,

        /// Dry run (show SQL without executing)
        #[arg(long)]
        dry_run: bool,
    },

    /// Rollback migrations
    Down {
        /// Number of migrations to rollback
        #[arg(short, long, default_value = "1")]
        count: usize,

        /// Dry run (show SQL without executing)
        #[arg(long)]
        dry_run: bool,
    },

    /// Show migration status
    Status,

    /// List all migrations
    List,

    /// Generate migration from model changes
    Makemigrations {
        /// App/module name
        #[arg(short, long)]
        app: Option<String>,

        /// Migration name
        #[arg(short, long)]
        name: Option<String>,

        /// Dry run (show changes without creating file)
        #[arg(long)]
        dry_run: bool,

        /// Auto-apply after generation
        #[arg(long)]
        auto: bool,
    },
}

#[derive(Subcommand)]
enum GenerateCommands {
    /// Generate models from database schema
    Models {
        /// Output directory
        #[arg(short, long, default_value = "src/models")]
        output: PathBuf,

        /// Tables to include (all if empty)
        #[arg(short, long)]
        tables: Vec<String>,

        /// Schema name
        #[arg(short, long)]
        schema: Option<String>,
    },

    /// Generate TypeScript types
    Types {
        /// Output file
        #[arg(short, long, default_value = "types.ts")]
        output: PathBuf,
    },
}

#[derive(Subcommand)]
enum SchemaCommands {
    /// Introspect database schema
    Introspect {
        /// Output format (json, toml, sql)
        #[arg(short, long, default_value = "json")]
        format: String,

        /// Output file (stdout if not specified)
        #[arg(short, long)]
        output: Option<PathBuf>,
    },

    /// Push schema changes to database
    Push {
        /// Dry run
        #[arg(long)]
        dry_run: bool,

        /// Accept data loss
        #[arg(long)]
        accept_data_loss: bool,
    },

    /// Pull schema from database
    Pull {
        /// Output file
        #[arg(short, long)]
        output: Option<PathBuf>,
    },

    /// Show schema diff
    Diff,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let cli = Cli::parse();

    // Initialize logging
    let log_level = if cli.verbose { "debug" } else { "info" };
    tracing_subscriber::registry()
        .with(tracing_subscriber::EnvFilter::try_from_default_env().unwrap_or_else(|_| log_level.into()))
        .with(tracing_subscriber::fmt::layer())
        .init();

    // Run command
    match cli.command {
        Commands::Init { path } => {
            commands::init::run(path).await?;
        }
        Commands::Db { command } => match command {
            DbCommands::Create => {
                commands::db::create(&cli.config, cli.database_url.as_deref()).await?;
            }
            DbCommands::Drop { force } => {
                commands::db::drop(&cli.config, cli.database_url.as_deref(), force).await?;
            }
            DbCommands::Reset { force } => {
                commands::db::reset(&cli.config, cli.database_url.as_deref(), force).await?;
            }
            DbCommands::Status => {
                commands::db::status(&cli.config, cli.database_url.as_deref()).await?;
            }
            DbCommands::Shell => {
                commands::db::shell(&cli.config, cli.database_url.as_deref()).await?;
            }
        },
        Commands::Migrate { command } => match command {
            MigrateCommands::New { name, app } => {
                commands::migrate::new(&cli.config, &name, app.as_deref()).await?;
            }
            MigrateCommands::Up { target, dry_run } => {
                commands::migrate::up(&cli.config, cli.database_url.as_deref(), target.as_deref(), dry_run)
                    .await?;
            }
            MigrateCommands::Down { count, dry_run } => {
                commands::migrate::down(&cli.config, cli.database_url.as_deref(), count, dry_run)
                    .await?;
            }
            MigrateCommands::Status => {
                commands::migrate::status(&cli.config, cli.database_url.as_deref()).await?;
            }
            MigrateCommands::List => {
                commands::migrate::list(&cli.config).await?;
            }
            MigrateCommands::Makemigrations { app, name, dry_run, auto } => {
                commands::migrate::makemigrations(&cli.config, cli.database_url.as_deref(), app.as_deref(), name.as_deref(), dry_run, auto)
                    .await?;
            }
        },
        Commands::Generate { command } => match command {
            GenerateCommands::Models { output, tables, schema } => {
                commands::generate::models(&cli.config, cli.database_url.as_deref(), &output, &tables, schema.as_deref())
                    .await?;
            }
            GenerateCommands::Types { output } => {
                commands::generate::types(&cli.config, cli.database_url.as_deref(), &output)
                    .await?;
            }
        },
        Commands::Schema { command } => match command {
            SchemaCommands::Introspect { format, output } => {
                commands::schema::introspect(&cli.config, cli.database_url.as_deref(), &format, output.as_deref())
                    .await?;
            }
            SchemaCommands::Push { dry_run, accept_data_loss } => {
                commands::schema::push(&cli.config, cli.database_url.as_deref(), dry_run, accept_data_loss)
                    .await?;
            }
            SchemaCommands::Pull { output } => {
                commands::schema::pull(&cli.config, cli.database_url.as_deref(), output.as_deref())
                    .await?;
            }
            SchemaCommands::Diff => {
                commands::schema::diff(&cli.config, cli.database_url.as_deref()).await?;
            }
        },
    }

    Ok(())
}
