//! Init command implementation

use colored::Colorize;
use std::path::Path;
use tokio::fs;

const DEFAULT_CONFIG: &str = r#"# Chakra ORM Configuration

[database]
# Database URL (can also be set via DATABASE_URL environment variable)
# url = "postgres://user:password@localhost:5432/mydb"

# Connection pool settings
[database.pool]
min_connections = 1
max_connections = 10
acquire_timeout = 30

[migrations]
# Migrations directory
path = "migrations"

[models]
# Models directory
path = "src/models"

[generate]
# Code generation settings
derive_debug = true
derive_clone = true
derive_serialize = true
"#;

const DEFAULT_GITIGNORE: &str = r#"# Chakra ORM
.chakra/
*.db
*.sqlite
"#;

pub async fn run(path: impl AsRef<Path>) -> Result<(), Box<dyn std::error::Error>> {
    let path = path.as_ref();

    println!("{}", "Initializing Chakra ORM project...".cyan().bold());

    // Create directories
    let migrations_dir = path.join("migrations");
    let models_dir = path.join("src/models");

    fs::create_dir_all(&migrations_dir).await?;
    fs::create_dir_all(&models_dir).await?;

    println!("  {} {}", "Created".green(), migrations_dir.display());
    println!("  {} {}", "Created".green(), models_dir.display());

    // Create config file
    let config_path = path.join("chakra.toml");
    if !config_path.exists() {
        fs::write(&config_path, DEFAULT_CONFIG).await?;
        println!("  {} {}", "Created".green(), config_path.display());
    } else {
        println!("  {} {} (already exists)", "Skipped".yellow(), config_path.display());
    }

    // Create mod.rs for models
    let mod_path = models_dir.join("mod.rs");
    if !mod_path.exists() {
        fs::write(&mod_path, "//! Database models\n").await?;
        println!("  {} {}", "Created".green(), mod_path.display());
    }

    // Append to .gitignore if it exists
    let gitignore_path = path.join(".gitignore");
    if gitignore_path.exists() {
        let content = fs::read_to_string(&gitignore_path).await?;
        if !content.contains(".chakra/") {
            let mut new_content = content;
            new_content.push_str("\n");
            new_content.push_str(DEFAULT_GITIGNORE);
            fs::write(&gitignore_path, new_content).await?;
            println!("  {} {}", "Updated".green(), gitignore_path.display());
        }
    }

    println!();
    println!("{}", "Project initialized successfully!".green().bold());
    println!();
    println!("Next steps:");
    println!("  1. Update {} with your database URL", "chakra.toml".cyan());
    println!("  2. Run {} to create your first migration", "chakra migrate new initial".cyan());
    println!("  3. Run {} to apply migrations", "chakra migrate up".cyan());

    Ok(())
}
